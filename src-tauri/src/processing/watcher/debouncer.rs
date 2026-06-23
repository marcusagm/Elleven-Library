use crate::core::events::payloads::DomainEvent;
use notify::{Event, EventKind};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::info;

/// Metadata snapshot used for heuristic rename pairing.
/// When a file is "added" (Created event), we capture its size and creation time
/// to later compare with removed files and detect untracked renames (macOS Finder).
#[derive(Debug, Clone)]
struct FileMetadataSnapshot {
    /// File size in bytes.
    size_bytes: u64,
    /// Creation timestamp (as raw `SystemTime` for comparison safety).
    created_at: Option<std::time::SystemTime>,
}

/// Internal state of the debouncer buffer.
#[derive(Debug)]
#[allow(dead_code)]
enum BufferedEvent {
    Created(Instant, Option<FileMetadataSnapshot>),
    Modified(Instant),
    /// Removed with a delayed confirmation guard.
    /// The `Instant` marks when the removal was first seen.
    Removed(Instant),
}

/// Debouncer Engine responsible for aggregating rapid filesystem events
/// and applying V1-quality rename heuristics before emitting clean Domain Events.
///
/// Ported heuristics from V1 `watcher.rs`:
/// - `RenameMode::Both` (Linux single-event renames)
/// - Tracker-based `From`/`To` pairing
/// - Metadata fallback pairing (size + created_at) for macOS Finder
/// - Delayed deletion guard (2s confirmation before emitting `FsPathDeleted`)
/// - File vs Directory classification
///
/// Ref: Sprint 10.2 — "Reconectar ao Heurístico de Rename da V1"
pub struct EventDebouncer {
    /// Channel to send processed domain events.
    output_sender: mpsc::Sender<DomainEvent>,
    /// Buffer of pending events with their last seen timestamp.
    buffer: HashMap<PathBuf, BufferedEvent>,
    /// Tracked renames: tracker_id → from_path (waiting for matching `To` event).
    pending_tracked_renames: HashMap<usize, PathBuf>,
    /// Untracked removes waiting for potential rename pairing.
    /// These have an extra grace period before being emitted as `FsPathDeleted`.
    pending_untracked_removes: HashMap<PathBuf, (Instant, Option<FileMetadataSnapshot>)>,
    /// History of recently emitted `FsFileDiscovered` events (path -> snapshot).
    /// Used for "late pairing" if a Remove event arrives after its Create counterpart was already emitted.
    recent_emitted_creates: HashMap<PathBuf, (Instant, FileMetadataSnapshot)>,
}

impl EventDebouncer {
    /// Create a new debouncer that sends output events to the given channel.
    pub fn new(output_sender: mpsc::Sender<DomainEvent>) -> Self {
        Self {
            output_sender,
            buffer: HashMap::new(),
            pending_tracked_renames: HashMap::new(),
            pending_untracked_removes: HashMap::new(),
            recent_emitted_creates: HashMap::new(),
        }
    }

    /// Process a single raw event from notify.
    ///
    /// This is Phase 1 of the V1 pipeline: "Parse and Normalize".
    /// Events are classified and buffered, with renames receiving special treatment.
    pub async fn handle_event(&mut self, event: Event) {
        let now = Instant::now();

        match event.kind {
            // ─── Gap 1 Fix: RenameMode::Both (Linux — single-event rename) ──
            EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::Both)) => {
                if event.paths.len() == 2 {
                    let from_path = event.paths[0].clone();
                    let to_path = event.paths[1].clone();

                    // Remove from buffers if the "from" was pending
                    self.buffer.remove(&from_path);
                    self.pending_untracked_removes.remove(&from_path);

                    // Emit rename immediately — Both events are reliable
                    let _ = self
                        .output_sender
                        .send(DomainEvent::FsPathRenamed {
                            from: from_path.to_string_lossy().to_string(),
                            to: to_path.to_string_lossy().to_string(),
                        })
                        .await;
                }
            }

            // ─── Tracked rename: From part ──────────────────────────────────
            EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::From)) => {
                if let Some(path) = event.paths.first() {
                    if let Some(tracker) = event.attrs.tracker() {
                        // Tracked rename — store and wait for matching `To`
                        self.pending_tracked_renames.insert(tracker, path.clone());
                    } else {
                        // Untracked remove — prepare for heuristic matching
                        let metadata_snapshot = read_metadata_snapshot(path);
                        self.pending_untracked_removes
                            .insert(path.clone(), (now, metadata_snapshot));
                        // Also remove from Created buffer if it was just added
                        self.buffer.remove(path);
                    }
                }
            }

            // ─── Tracked rename: To part ────────────────────────────────────
            EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::To)) => {
                if let Some(path) = event.paths.first() {
                    let matched_from = match event.attrs.tracker() {
                        Some(tracker) => self.pending_tracked_renames.remove(&tracker),
                        None => None,
                    };

                    if let Some(from_path) = matched_from {
                        // Tracked rename pair found — emit
                        self.buffer.remove(&from_path);
                        let _ = self
                            .output_sender
                            .send(DomainEvent::FsPathRenamed {
                                from: from_path.to_string_lossy().to_string(),
                                to: path.to_string_lossy().to_string(),
                            })
                            .await;
                    } else {
                        // No tracker match — buffer as Created for heuristic pairing
                        let metadata_snapshot = read_metadata_snapshot(path);
                        self.buffer
                            .insert(path.clone(), BufferedEvent::Created(now, metadata_snapshot));
                    }
                }
            }

            // ─── Create events ──────────────────────────────────────────────
            EventKind::Create(_) => {
                for path in event.paths {
                    let metadata_snapshot = read_metadata_snapshot(&path);
                    self.buffer
                        .insert(path, BufferedEvent::Created(now, metadata_snapshot));
                }
            }

            // ─── Modify events ──────────────────────────────────────────────
            EventKind::Modify(_) => {
                for path in event.paths {
                    self.buffer.insert(path, BufferedEvent::Modified(now));
                }
            }

            // ─── Remove events ──────────────────────────────────────────────
            EventKind::Remove(_) => {
                for path in event.paths {
                    // Gap 3 Fix: Don't emit immediately — put in delayed confirmation
                    let metadata_snapshot = read_metadata_snapshot(&path);
                    self.pending_untracked_removes
                        .insert(path.clone(), (now, metadata_snapshot));
                    // Remove from Created buffer if it was just added
                    self.buffer.remove(&path);
                }
            }

            _ => {}
        }
    }

    /// Tick function to check for expired debouncing windows.
    ///
    /// This is Phase 2-3 of the V1 pipeline: "Classification and Heuristics".
    /// It also handles the delayed deletion guard.
    pub async fn tick(&mut self, debounce_window: Duration) {
        let now = Instant::now();
        let deletion_guard_duration = Duration::from_secs(3);

        // ─── Process expired pending_tracked_renames (orphaned From events) ─
        let orphaned_trackers: Vec<usize> = self.pending_tracked_renames.keys().cloned().collect();
        for tracker_id in orphaned_trackers {
            // If a tracked From has been sitting for too long without a To,
            // treat it as a deletion
            if let Some(from_path) = self.pending_tracked_renames.remove(&tracker_id) {
                let metadata_snapshot = read_metadata_snapshot(&from_path);
                self.pending_untracked_removes
                    .insert(from_path, (now, metadata_snapshot));
            }
        }

        // 6. Prune recent emitted creates history (5s expiry to cover late renames)
        self.recent_emitted_creates
            .retain(|_, (instant, _)| instant.elapsed() < Duration::from_secs(5));

        // ─── Gap 3 Fix: Delayed deletion guard ─────────────────────────
        let mut confirmed_deletions: Vec<PathBuf> = Vec::new();
        let mut paths_to_remove_from_pending: Vec<PathBuf> = Vec::new();

        for (path, (first_seen, _metadata_snapshot)) in &self.pending_untracked_removes {
            if now.duration_since(*first_seen) >= deletion_guard_duration {
                paths_to_remove_from_pending.push(path.clone());

                // Re-verify: if the path has reappeared (rename completed), don't delete
                if !path_exists_exact(path) {
                    confirmed_deletions.push(path.clone());
                } else {
                    // Path reappeared — this was a temporary disappearance.
                    // Buffer it as a Created event for the normal debounce path
                    let snapshot = read_metadata_snapshot(path);
                    self.buffer
                        .insert(path.clone(), BufferedEvent::Created(now, snapshot));
                }
            }
        }

        for path in &paths_to_remove_from_pending {
            self.pending_untracked_removes.remove(path);
        }

        for path in &confirmed_deletions {
            self.pending_untracked_removes.remove(path);
            // Gap 4 Fix: classify as directory vs file
            let domain_event = if path.is_dir() || is_likely_directory(path) {
                DomainEvent::FsDirectoryDeleted {
                    path: path.to_string_lossy().to_string(),
                }
            } else {
                DomainEvent::FsPathDeleted {
                    path: path.to_string_lossy().to_string(),
                }
            };
            let _ = self.output_sender.send(domain_event).await;
        }

        let mut expired_paths = Vec::new();
        for (path, event) in &self.buffer {
            let last_seen = match event {
                BufferedEvent::Created(timestamp, _) => timestamp,
                BufferedEvent::Modified(timestamp) => timestamp,
                BufferedEvent::Removed(timestamp) => timestamp,
            };

            if now.duration_since(*last_seen) >= debounce_window {
                expired_paths.push(path.clone());
            }
        }

        // Pass 1: Move expired paths that no longer exist to pending_untracked_removes
        let mut surviving_paths = Vec::new();
        for path in expired_paths {
            if !path_exists_exact(&path) {
                if let Some(event) = self.buffer.remove(&path) {
                    let snapshot = match event {
                        BufferedEvent::Created(_, meta) => meta,
                        _ => None, // Cannot read metadata for a deleted file now
                    };
                    self.pending_untracked_removes
                        .insert(path, (now, snapshot));
                }
            } else {
                surviving_paths.push(path);
            }
        }

        // ─── Gap 2 Fix: Heuristic pairing for untracked renames ─────────
        // Now that deletions are in pending_untracked_removes, we can try to pair them
        // with the surviving creations/modifications in the buffer.
        self.apply_rename_heuristics().await;

        // Pass 2: Emit discovery events for paths that survived the heuristic pairing
        for path in surviving_paths {
            if let Some(event) = self.buffer.remove(&path) {
                // Double check existence just in case
                if !path_exists_exact(&path) {
                    continue;
                }

                let path_str = path.to_string_lossy().to_string();
                let domain_event = match event {
                    BufferedEvent::Created(_, _) | BufferedEvent::Modified(_) => {
                        // Gap 4 Fix: classify as directory vs file
                        if path.is_dir() {
                            DomainEvent::FsDirectoryDiscovered { path: path_str }
                        } else {
                            let size = std::fs::metadata(&path)
                                .map(|metadata| metadata.len())
                                .unwrap_or(0);

                            // Record in recent emitted creates for late pairing fallback
                            if let Some(meta) = read_metadata_snapshot(&path) {
                                self.recent_emitted_creates
                                    .insert(path.clone(), (Instant::now(), meta));
                            }

                            DomainEvent::FsFileDiscovered {
                                path: path_str,
                                size_bytes: size,
                            }
                        }
                    }
                    BufferedEvent::Removed(_) => {
                        // This shouldn't normally happen (removals go through
                        // pending_untracked_removes now), but handle gracefully
                        DomainEvent::FsPathDeleted { path: path_str }
                    }
                };
                let _ = self.output_sender.send(domain_event).await;
            }
        }
    }

    /// Gap 2 Fix: Heuristic rename pairing using metadata.
    ///
    /// Port of V1 `phase_classify_heuristics` (watcher.rs lines 226-272).
    /// For each pending untracked remove, look for a newly Created file with
    /// matching `size + created_at` → treat as rename pair.
    async fn apply_rename_heuristics(&mut self) {
        let mut renames_found: Vec<(PathBuf, PathBuf)> = Vec::new();

        let remove_paths: Vec<PathBuf> = self.pending_untracked_removes.keys().cloned().collect();

        for from_path in remove_paths {
            let from_meta = self
                .pending_untracked_removes
                .get(&from_path)
                .and_then(|(_, m)| m.clone());
            let mut matched_to_path = None;

            // 1. Try strict matching (Size + CreatedAt) if metadata is available
            // ALLOW cross-folder matches if it's a direct metadata hit
            if let Some(meta) = &from_meta {
                matched_to_path = self
                    .buffer
                    .iter()
                    .find(|(to_path, event)| {
                        if **to_path == from_path {
                            return false;
                        }
                        if let BufferedEvent::Created(_, Some(created_snapshot)) = event {
                            created_snapshot.size_bytes == meta.size_bytes
                                && created_snapshot.created_at == meta.created_at
                        } else {
                            false
                        }
                    })
                    .map(|(path, _)| path.clone());
            }

            // 2. Fallback matching (Buffer): Same Parent + Same Extension
            if matched_to_path.is_none() {
                matched_to_path = self
                    .buffer
                    .iter()
                    .find(|(to_path, event)| {
                        **to_path != from_path
                            && (matches!(event, BufferedEvent::Created(_, _)) || matches!(event, BufferedEvent::Modified(_)))
                            && to_path.parent() == from_path.parent()
                            && to_path.extension() == from_path.extension()
                            && is_likely_directory(&from_path) == is_likely_directory(to_path)
                    })
                    .map(|(path, _)| path.clone());
            }

            // 3. Fallback matching (Recent Emitted): Same Parent + Same Extension
            // (Crucial for macOS where Metadata might be missing for the "from" path)
            if matched_to_path.is_none() {
                for (to_path, (_instant, to_meta)) in &self.recent_emitted_creates {
                    if *to_path == from_path {
                        continue;
                    }
                    let ext_match = from_path.extension() == to_path.extension() 
                        && is_likely_directory(&from_path) == is_likely_directory(to_path);

                    if ext_match {
                        // If we have metadata, use it to confirm
                        if let (Some(f_meta), t_meta) = (&from_meta, to_meta) {
                            if f_meta.size_bytes == t_meta.size_bytes {
                                info!(
                                    "Debouncer: Late Heuristic MATCH (Meta): {} -> {}",
                                    from_path.display(),
                                    to_path.display()
                                );
                                matched_to_path = Some(to_path.clone());
                                break;
                            }
                        } else if from_path.parent() == to_path.parent() && is_likely_directory(&from_path) == is_likely_directory(to_path) {
                            // No metadata for 'from', but path looks like a rename candidate in same folder
                            info!(
                                "Debouncer: Late Heuristic MATCH (Path-only): {} -> {}",
                                from_path.display(),
                                to_path.display()
                            );
                            matched_to_path = Some(to_path.clone());
                            break;
                        }
                    }
                }
            }

            if let Some(to_path) = matched_to_path {
                info!(
                    "Debouncer: Heuristic MATCH found: {} -> {}",
                    from_path.display(),
                    to_path.display()
                );
                self.buffer.remove(&to_path);
                renames_found.push((from_path, to_path));
            }
        }

        // Emit rename events and clean up buffers
        for (from_path, to_path) in renames_found {
            self.pending_untracked_removes.remove(&from_path);
            let _ = self
                .output_sender
                .send(DomainEvent::FsPathRenamed {
                    from: from_path.to_string_lossy().to_string(),
                    to: to_path.to_string_lossy().to_string(),
                })
                .await;
        }
    }
}

/// Reads a lightweight metadata snapshot from the filesystem.
/// Returns `None` if the file/path doesn't exist or metadata can't be read.
fn read_metadata_snapshot(path: &PathBuf) -> Option<FileMetadataSnapshot> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileMetadataSnapshot {
        size_bytes: metadata.len(),
        created_at: metadata.created().ok(),
    })
}

/// Fallback function to determine if a path is likely a directory.
fn is_likely_directory(path: &std::path::Path) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        return metadata.is_dir();
    }
    
    // Hidden files (dotfiles) like .DS_Store are usually not directories
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return false;
        }
    }
    
    path.extension().is_none()
}

/// Checks if a path exists with the exact case specified.
/// This is crucial for macOS, which is case-insensitive by default.
/// If we just use `path.exists()`, renaming "subpasta" to "Subpasta" will appear
/// as if "subpasta" still exists, breaking the rename heuristic.
fn path_exists_exact(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    if let Some(file_name) = path.file_name() {
        if let Some(parent) = path.parent() {
            if let Ok(mut entries) = std::fs::read_dir(parent) {
                while let Some(Ok(entry)) = entries.next() {
                    if entry.file_name() == file_name {
                        return true;
                    }
                }
            }
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{ModifyKind, RenameMode};

    #[tokio::test]
    async fn test_event_aggregation() {
        let (sender, mut receiver) = mpsc::channel(100);
        let mut debouncer = EventDebouncer::new(sender);

        // Create a real temp file so `path.exists()` is true
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("test_event_agg_{}.txt", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
        std::fs::write(&path, "test content").unwrap();

        let event = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(path.clone());

        // Send 3 events in rapid succession
        debouncer.handle_event(event.clone()).await;
        debouncer.handle_event(event.clone()).await;
        debouncer.handle_event(event.clone()).await;

        assert_eq!(debouncer.buffer.len(), 1);

        // Tick with small window (should not emit)
        debouncer.tick(Duration::from_millis(500)).await;
        assert!(receiver.try_recv().is_err());

        // Tick with expired window
        tokio::time::sleep(Duration::from_millis(600)).await;
        debouncer.tick(Duration::from_millis(500)).await;

        let result = receiver
            .try_recv()
            .expect("Should have received a domain event");
        match result {
            DomainEvent::FsFileDiscovered {
                path: event_path, ..
            } => {
                assert_eq!(event_path, path.to_string_lossy().to_string());
            }
            _ => panic!("Expected FsFileDiscovered"),
        }

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_rename_mode_both() {
        let (sender, mut receiver) = mpsc::channel(100);
        let mut debouncer = EventDebouncer::new(sender);

        let from = PathBuf::from("/tmp/old.txt");
        let to = PathBuf::from("/tmp/new.txt");

        let event = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(from.clone())
            .add_path(to.clone());

        debouncer.handle_event(event).await;

        let result = receiver
            .try_recv()
            .expect("Should have received a rename event");
        match result {
            DomainEvent::FsPathRenamed {
                from: event_from,
                to: event_to,
            } => {
                assert_eq!(event_from, from.to_string_lossy().to_string());
                assert_eq!(event_to, to.to_string_lossy().to_string());
            }
            _ => panic!("Expected FsPathRenamed"),
        }
    }

    #[tokio::test]
    async fn test_delayed_deletion_guard() {
        let (sender, mut receiver) = mpsc::channel(100);
        let mut debouncer = EventDebouncer::new(sender);

        let path = PathBuf::from("/tmp/nonexistent_test_file_12345.txt");
        let event =
            Event::new(EventKind::Remove(notify::event::RemoveKind::File)).add_path(path.clone());

        debouncer.handle_event(event).await;

        // Immediate tick — should NOT emit (guard period not expired)
        debouncer.tick(Duration::from_millis(200)).await;
        assert!(
            receiver.try_recv().is_err(),
            "Should NOT have emitted yet — deletion guard active"
        );

        // Wait for guard period to expire (debouncer uses 3s deletion guard)
        tokio::time::sleep(Duration::from_secs(4)).await;
        debouncer.tick(Duration::from_millis(200)).await;

        // Now it should emit (path doesn't exist)
        let result = receiver
            .try_recv()
            .expect("Should have received a deletion event after guard");
        match result {
            DomainEvent::FsPathDeleted { path: event_path } => {
                assert_eq!(event_path, path.to_string_lossy().to_string());
            }
            _ => panic!("Expected FsPathDeleted"),
        }
    }
}
