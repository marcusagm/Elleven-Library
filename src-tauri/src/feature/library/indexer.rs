use crate::core::error::AppResult;
use crate::core::events::AppEventBus;
use crate::core::ledger::command::{CreateAssetPayload, CreateFolderPayload, LedgerCommand};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::AssetState;
use crate::core::repository::AssetQueryHandler;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinSet;

use tracing::{debug, error, info, instrument, warn};
use walkdir::{DirEntry, WalkDir};

/// Intermediate result produced by each fan-out producer task.
enum AssetDiscoveryResult {
    /// A new or modified asset was detected and needs to be persisted.
    NewAsset(CreateAssetPayload),
    /// The file was already indexed and unchanged — skip.
    ExistingAsset,
    /// A parsing or I/O error occurred — skip this file gracefully.
    Error(String),
}

/// Service responsible for indexing library folders using a parallel
/// producer-consumer pipeline.
///
/// Ref: Sprint 10.2 — "Fan-out Producer-Consumer"
pub struct LibraryIndexer {
    /// Port for read-only asset operations (differential cache).
    query_handler: Arc<dyn AssetQueryHandler>,
    /// Port for state mutations.
    ledger: Arc<dyn TransactionalAssetLedger>,
    /// Event bus for publishing domain events.
    event_bus: Arc<dyn AppEventBus>,
    /// The central "Cartório" for format definitions.
    registry: Arc<crate::core::formats::registry::FormatRegistry>,
    /// Maximum number of concurrent file-processing tasks.
    concurrency_limit: usize,

    /// Cache of recently removed files metadata to support "Implicit Rename" recovery.
    /// Keeps track of removals for a short window so that a subsequent file discovery
    /// with matching size + created_at can be paired as a move instead of delete+create.
    /// Tuple: (removal_timestamp, file_size, created_at_from_db)
    recent_removals: Arc<DashMap<PathBuf, (DateTime<Utc>, u64, Option<DateTime<Utc>>)>>,
}

impl LibraryIndexer {
    /// Create a new LibraryIndexer.
    pub fn new(
        query_handler: Arc<dyn AssetQueryHandler>,
        ledger: Arc<dyn TransactionalAssetLedger>,
        event_bus: Arc<dyn AppEventBus>,
        registry: Arc<crate::core::formats::registry::FormatRegistry>,
    ) -> Self {
        Self {
            query_handler,
            ledger,
            event_bus,
            registry,
            concurrency_limit: 200,
            recent_removals: Arc::new(DashMap::new()),
        }
    }

    /// Create a new LibraryIndexer with a custom concurrency limit.
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit.max(10).min(500);
        self
    }

    /// Perform a parallel differential scan of a directory tree.
    ///
    /// Pipeline:
    /// 1. Single `spawn_blocking` walk → collect all entries
    /// 2. Process folders hierarchically (sequential, before fan-out)
    /// 3. Fan-out file classification via `JoinSet` + `Semaphore`
    /// 4. Consumer serializes persistence via `Ledger.BatchCreate`
    #[instrument(skip(self), fields(path = %path.display()))]
    pub async fn scan_directory(&self, path: PathBuf, folder_id: Option<String>) -> AppResult<()> {
        let scan_start_time = std::time::Instant::now();
        let root_str = path.to_string_lossy().to_string();
        info!(
            "▶ Scan STARTED for: {} (concurrency_limit={})",
            root_str, self.concurrency_limit
        );

        // Resolve folder_id if not provided
        let current_root_id = if let Some(id) = folder_id {
            Some(id)
        } else {
            self.query_handler.find_folder_by_path(&root_str).await?
        };

        // Emit ScanStarted
        let _ = self
            .event_bus
            .publish(crate::core::events::DomainEvent::ScanStarted {
                library_id: current_root_id
                    .clone()
                    .unwrap_or_else(|| "root".to_string()),
            });

        // ─── PHASE 1: Single Walk (eliminates duplicate WalkDir) ──────────
        let registry_for_walk = self.registry.clone();
        let walk_path = path.clone();
        let all_entries: Vec<DirEntry> = tokio::task::spawn_blocking(move || {
            WalkDir::new(&walk_path)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .collect()
        })
        .await
        .map_err(|join_error| {
            crate::core::error::AppError::Internal(format!("WalkDir join failed: {}", join_error))
        })?;

        // Separate directories and files
        let mut directory_entries: Vec<DirEntry> = Vec::new();
        let mut file_entries: Vec<DirEntry> = Vec::new();

        for entry in all_entries {
            if entry.file_type().is_dir() {
                if entry.path() != path {
                    directory_entries.push(entry);
                }
            } else if entry.file_type().is_file() {
                let is_supported = entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| registry_for_walk.is_supported_extension(ext))
                    .unwrap_or(false);

                if is_supported {
                    file_entries.push(entry);
                }
            }
        }

        let total_files = file_entries.len();
        debug!(
            "Walk completed: {} supported files, {} subdirectories",
            total_files,
            directory_entries.len()
        );

        // ─── PHASE 2: Pre-load folder cache + ensure hierarchy ────────────
        // Load existing folders into a HashMap for O(1) lookup
        let existing_folders = self.query_handler.list_all_subfolders().await?;
        let folder_cache: Arc<RwLock<HashMap<PathBuf, String>>> = Arc::new(RwLock::new({
            let mut map = HashMap::new();
            for folder in &existing_folders {
                map.insert(folder.path.clone(), folder.id.clone());
            }
            // Ensure root is in cache
            if let Some(ref root_id) = current_root_id {
                map.insert(path.clone(), root_id.clone());
            }
            map
        }));

        // Sort directories by depth (shortest path first = parents before children)
        directory_entries.sort_by_key(|entry| entry.path().components().count());

        for dir_entry in &directory_entries {
            let dir_path = dir_entry.path().to_path_buf();
            let dir_path_str = dir_path.to_string_lossy().to_string();

            // Check if folder already exists in cache
            {
                let cache_read = folder_cache.read().await;
                if cache_read.contains_key(&dir_path) {
                    continue;
                }
            }

            // Check DB
            if let Some(existing_id) = self
                .query_handler
                .find_folder_by_path(&dir_path_str)
                .await?
            {
                let mut cache_write = folder_cache.write().await;
                cache_write.insert(dir_path, existing_id);
                continue;
            }

            // Create folder — parent must already be in cache (sorted by depth)
            let parent_path = dir_path.parent().unwrap_or(&path);
            let parent_id = {
                let cache_read = folder_cache.read().await;
                cache_read.get(parent_path).cloned()
            };

            let name = dir_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let create_folder_command = LedgerCommand::CreateFolder(CreateFolderPayload {
                parent_id,
                name,
                path: dir_path.clone(),
            });

            match self.ledger.execute(create_folder_command).await {
                Ok(folder_asset) => {
                    let mut cache_write = folder_cache.write().await;
                    cache_write.insert(dir_path, folder_asset.id);
                }
                Err(error) => {
                    warn!("Failed to create folder {:?}: {}", dir_path, error);
                }
            }
        }

        // ─── PHASE 3: Load comparison cache for differential check ────────
        let comparison_cache = self
            .query_handler
            .get_all_files_comparison_data(&root_str)
            .await?;
        debug!(
            "Loaded {} entries from comparison cache",
            comparison_cache.len()
        );

        // ─── PHASE 4: Fan-out file classification ─────────────────────────
        let (result_sender, mut result_receiver) = mpsc::channel::<AssetDiscoveryResult>(2000);
        let mut producer_join_set = JoinSet::new();
        let concurrency_semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        // Share read-only data across producers
        let shared_comparison_cache = Arc::new(comparison_cache);
        let shared_folder_cache = folder_cache.clone();
        let shared_registry = self.registry.clone();

        // Pre-calculate verified paths for Phase 6 before consuming entries
        let verified_file_paths: std::collections::HashSet<String> = file_entries
            .iter()
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        let verified_folder_paths: std::collections::HashSet<String> = directory_entries
            .iter()
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        for file_entry in file_entries {
            let sender_clone = result_sender.clone();
            let semaphore_clone = concurrency_semaphore.clone();
            let cache_clone = shared_comparison_cache.clone();
            let folder_cache_clone = shared_folder_cache.clone();
            let registry_clone = shared_registry.clone();
            let root_id_clone = current_root_id.clone();

            producer_join_set.spawn(async move {
                // Acquire semaphore permit — limits concurrent tasks
                let _permit = match semaphore_clone.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => return, // Semaphore closed
                };

                let result = classify_file_entry(
                    &file_entry,
                    &cache_clone,
                    &folder_cache_clone,
                    &registry_clone,
                    &root_id_clone,
                )
                .await;

                let _ = sender_clone.send(result).await;
            });
        }
        // Close the sender so the consumer knows when all producers are done
        drop(result_sender);

        // ─── PHASE 5: Consumer — serialize persistence via Ledger ─────────
        let mut batch_payloads: Vec<CreateAssetPayload> = Vec::new();
        let mut discovered_count: u64 = 0;
        let mut unchanged_count: u64 = 0;
        let mut error_count: u64 = 0;
        let mut processed_count: u64 = 0;
        let batch_size: usize = 100;

        while let Some(discovery) = result_receiver.recv().await {
            processed_count += 1;

            match discovery {
                AssetDiscoveryResult::NewAsset(payload) => {
                    batch_payloads.push(payload);
                    discovered_count += 1;

                    // Flush batch when it reaches the threshold
                    if batch_payloads.len() >= batch_size {
                        if let Err(batch_error) = self
                            .ledger
                            .execute(LedgerCommand::BatchCreate(std::mem::take(
                                &mut batch_payloads,
                            )))
                            .await
                        {
                            error!("BatchCreate failed: {}", batch_error);
                        }
                    }
                }
                AssetDiscoveryResult::ExistingAsset => {
                    unchanged_count += 1;
                }
                AssetDiscoveryResult::Error(error_message) => {
                    warn!("Scan: skipping file: {}", error_message);
                    error_count += 1;
                }
            }

            // Emit progress periodically
            if processed_count.is_multiple_of(100) || processed_count == total_files as u64 {
                let _ = self
                    .event_bus
                    .publish(crate::core::events::DomainEvent::ScanProgress {
                        total: total_files,
                        processed: processed_count as usize,
                        current_file: format!("{}/{}", processed_count, total_files),
                    });
            }
        }

        // Flush remaining batch
        if !batch_payloads.is_empty() {
            if let Err(batch_error) = self
                .ledger
                .execute(LedgerCommand::BatchCreate(batch_payloads))
                .await
            {
                error!("Final BatchCreate failed: {}", batch_error);
            }
        }

        // Wait for all producers to finish (they should already be done since channel is drained)
        while producer_join_set.join_next().await.is_some() {}

        // ─── PHASE 6: Prune Stale Assets and Folders ─────────────────────────
        let mut pruned_files_count = 0;
        let mut pruned_folders_count = 0;

        // (Verified paths collected before fan-out)
        // Prune Missing Files
        for cached_path in shared_comparison_cache.keys() {
            if !verified_file_paths.contains(cached_path) {
                if let Err(e) = self
                    .ledger
                    .execute(LedgerCommand::DeleteAsset {
                        asset_id: None,
                        path: Some(PathBuf::from(cached_path)),
                        physical_delete: false,
                    })
                    .await
                {
                    warn!("Failed to prune stale file {}: {}", cached_path, e);
                } else {
                    pruned_files_count += 1;
                }
            }
        }

        // Prune Missing Folders
        for folder in existing_folders {
            let folder_path_str = folder.path.to_string_lossy().to_string();

            if folder_path_str.starts_with(&root_str)
                && folder_path_str != root_str
                && !verified_folder_paths.contains(&folder_path_str)
            {
                if let Err(e) = self
                    .ledger
                    .execute(LedgerCommand::RemoveFolder(
                        crate::core::ledger::command::RemoveFolderPayload {
                            folder_id: folder.id.clone(),
                        },
                    ))
                    .await
                {
                    warn!("Failed to prune stale folder {}: {}", folder_path_str, e);
                } else {
                    pruned_folders_count += 1;
                }
            }
        }

        let scan_duration = scan_start_time.elapsed();
        info!(
            "■ Scan COMPLETED for: {} — Discovered: {}, Unchanged: {}, Pruned: {} ({} files/{} dirs), Errors: {}, Duration: {:.2}s",
            root_str, discovered_count, unchanged_count, pruned_files_count + pruned_folders_count, pruned_files_count, pruned_folders_count, error_count, scan_duration.as_secs_f64()
        );

        // Emit ScanCompleted
        let _ = self
            .event_bus
            .publish(crate::core::events::DomainEvent::ScanCompleted {
                library_id: current_root_id.unwrap_or_else(|| "root".to_string()),
            });

        Ok(())
    }

    /// Starts a background loop that listens for domain events and handles
    /// individual file system changes through direct ledger commands.
    ///
    /// This mirrors the V1 behavior where each event (add, rename, delete)
    /// was processed as a single, targeted operation — NOT a full re-scan.
    pub async fn start_event_listener(self: Arc<Self>, event_bus: Arc<dyn AppEventBus>) {
        let mut receiver = event_bus.subscribe();
        tokio::spawn(async move {
            info!("IndexerEventListener: started and listening for filesystem events");

            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        self.handle_single_event(event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped_count)) => {
                        warn!(
                            "IndexerEventListener: lagged behind {} events, continuing",
                            skipped_count
                        );
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("IndexerEventListener: channel closed, shutting down");
                        break;
                    }
                }
            }
        });
    }

    /// Handle a single DomainEvent with a targeted ledger operation.
    ///
    /// Instead of calling scan_directory() (which walks 42k+ files), this
    /// performs the minimal database operation for each event type.
    async fn handle_single_event(&self, event: crate::core::events::DomainEvent) {
        use crate::core::events::DomainEvent;

        match event {
            DomainEvent::FsFileDiscovered { path, .. } => {
                self.handle_file_discovered(path).await;
            }

            DomainEvent::FsPathDeleted { path } => {
                self.handle_path_deleted(path).await;
            }

            DomainEvent::FsPathRenamed { from, to } => {
                self.handle_path_renamed(from, to).await;
            }

            DomainEvent::FsDirectoryDiscovered { path } => {
                let dir_path = PathBuf::from(&path);
                let dir_path_str = dir_path.to_string_lossy().to_string();

                if let Ok(Some(_)) = self.query_handler.find_folder_by_path(&dir_path_str).await {
                    return;
                }

                let parent_id = if let Some(parent) = dir_path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    self.query_handler
                        .find_folder_by_path(&parent_str)
                        .await
                        .unwrap_or(None)
                } else {
                    None
                };

                let folder_name = dir_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let create_result = self
                    .ledger
                    .execute(LedgerCommand::CreateFolder(CreateFolderPayload {
                        parent_id: parent_id.clone(),
                        name: folder_name,
                        path: dir_path.clone(),
                    }))
                    .await;
                
                // Immediately scan the new folder to discover any files that were dragged/restored inside it
                if let Ok(new_folder_asset) = create_result {
                    let _ = self.scan_directory(dir_path, Some(new_folder_asset.id)).await;
                } else {
                    // Fallback to scanning with parent_id if folder creation mysteriously failed but we still want to scan
                    let _ = self.scan_directory(dir_path, parent_id).await;
                }
            }
            DomainEvent::FsDirectoryDeleted { path } => {
                if let Ok(Some(folder_id)) = self.query_handler.find_folder_by_path(&path).await {
                    let _ = self
                        .ledger
                        .execute(LedgerCommand::RemoveFolder(
                            crate::core::ledger::command::RemoveFolderPayload { folder_id },
                        ))
                        .await;
                }
            }

            _ => {}
        }
    }

    /// Handles a new file discovery event, potentially recovering a rename or move.
    async fn handle_file_discovered(&self, path: String) {
        let entry_path = PathBuf::from(&path);

        // 1. Strategic Delay: On macOS, a Rename is often emitted as Delete then Create.
        // 500ms gives enough time for handle_path_deleted to populate recent_removals.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Try to get metadata for fingerprint matching
        let disk_metadata = std::fs::metadata(&entry_path).ok();
        let disk_size = disk_metadata.as_ref().map(|metadata| metadata.len() as i64).unwrap_or(0);
        let disk_created_at: Option<DateTime<Utc>> = disk_metadata
            .as_ref()
            .and_then(|metadata| metadata.created().ok())
            .map(DateTime::<Utc>::from);

        // --- STEP 1: Fast Match (Size + CreatedAt from recent_removals) ---
        let from_path = self
            .recent_removals
            .iter()
            .find(|entry| {
                if *entry.key() == entry_path {
                    return false; // Not a move if it's the exact same path
                }
                
                let (_, (_, removed_size, removed_created_at)) = entry.pair();
                let size_matches = *removed_size as i64 == disk_size && disk_size > 0;
                if !size_matches {
                    return false;
                }
                // If both sides have created_at, require a match for precision
                match (removed_created_at, &disk_created_at) {
                    (Some(database_created), Some(filesystem_created)) => {
                        (*database_created - *filesystem_created).num_seconds().abs() < 2
                    }
                    _ => true, // If either side lacks created_at, rely on size alone
                }
            })
            .map(|entry| entry.key().clone());

        if let Some(from_path) = from_path {
            info!(
                "Indexer: Fast Match (Size: {}) - Treating as Move: {} -> {}",
                disk_size,
                from_path.to_string_lossy(),
                path
            );

            self.recent_removals.remove(&from_path);

            let update_payload = crate::core::ledger::command::UpdateAssetPayload {
                asset_id: None,
                old_path: Some(from_path),
                new_path: entry_path.clone(),
            };
            let _ = self
                .ledger
                .execute(LedgerCommand::UpdateAsset(update_payload))
                .await;
            return;
        }

        // --- STEP 2: Normal Discovery (Creation with Collision Check) ---
        let _ = self.handle_file_discovery_event(entry_path).await;
    }

    /// Extacts metadata and persists a new file to the ledger.
    async fn handle_file_discovery_event(&self, entry_path: PathBuf) -> AppResult<()> {
        let path_str = entry_path.to_string_lossy().to_string();

        // CRITICAL: Existence check first (Race Condition Protection)
        // If the path already exists in the database, don't create it again.
        // This prevents duplicates if FsFileDiscovered was already emitted or handled.
        if let Ok(Some(_)) = self.query_handler.find_asset_by_path(&path_str).await {
            debug!(
                "Indexer: Asset already exists for path '{}', skipping duplicate creation",
                entry_path.display()
            );
            return Ok(());
        }

        let extension = entry_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if !self.registry.is_supported_extension(extension) {
            return Ok(());
        }

        let (format_name, family_name) = self
            .registry
            .detect(&entry_path)
            .map(|f| (f.name.to_string(), f.type_category.to_string()))
            .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

        let metadata = std::fs::metadata(&entry_path).map_err(crate::core::error::AppError::Io)?;
        let size_bytes = metadata.len();
        let created_at = metadata.created().ok().map(DateTime::<Utc>::from);
        let modified_at = metadata.modified().ok().map(DateTime::<Utc>::from);

        let folder_id = if let Some(parent) = entry_path.parent() {
            self.ensure_folder_hierarchy(parent).await?
        } else {
            None
        };

        let create_payload = CreateAssetPayload {
            path: entry_path,
            file_size: size_bytes,
            format_type: format_name,
            family: family_name,
            state_init: AssetState::Indexed,
            folder_id,
            created_at,
            modified_at,
        };

        self.ledger
            .execute(LedgerCommand::CreateAsset(create_payload))
            .await?;
        Ok(())
    }

    /// Handles a path deletion event.
    ///
    /// The debouncer has already applied a 3-second deletion guard before emitting
    /// FsPathDeleted. The indexer trusts this judgment and executes immediately,
    /// but maintains a `recent_removals` cache so that `handle_file_discovered`
    /// can still pair late renames/moves.
    async fn handle_path_deleted(&self, path: String) {
        let entry_path = PathBuf::from(&path);
        info!("Indexer: Processing DELETE for: {}", path);

        // Capture size AND created_at from DB for precise move pairing
        let mut old_size = 0;
        let mut old_created_at: Option<DateTime<Utc>> = None;
        if let Ok(Some(asset)) = self.query_handler.find_asset_by_path(&path).await {
            old_size = asset.file_size;
            old_created_at = asset.created_at;
        }

        // Populate recent_removals cache for potential late rename/move pairing
        self.recent_removals
            .insert(entry_path.clone(), (Utc::now(), old_size, old_created_at));

        // Execute delete immediately — the debouncer already confirmed the file is gone
        let delete_command = LedgerCommand::DeleteAsset {
            asset_id: None,
            path: Some(entry_path.clone()),
            physical_delete: false,
        };

        if let Err(error) = self.ledger.execute(delete_command).await {
            // NotFound is expected for files not in the DB (e.g., unsupported formats)
            if !matches!(error, crate::core::error::AppError::NotFound(_)) {
                error!("Indexer: DELETE failed for {}: {}", path, error);
            }
        }

        // Clean up recent_removals after 5 seconds (enough time for late move pairing)
        let recent_removals_clone = self.recent_removals.clone();
        let entry_path_clone = entry_path.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            recent_removals_clone.remove(&entry_path_clone);
        });
    }

    /// Handles an explicit path rename, cleaning up rename caches for the source.
    async fn handle_path_renamed(&self, from: String, to: String) {
        let from_path = PathBuf::from(&from);
        let to_path = PathBuf::from(&to);

        // Clean up rename pairing cache
        self.recent_removals.remove(&from_path);

        if to_path.is_dir() {
            let found_folder = self
                .query_handler
                .find_folder_by_path(&from_path.to_string_lossy())
                .await
                .unwrap_or(None);

            if let Some(folder_id) = found_folder {
                // Normal rename: old path exists in DB, update it
                let _ = self
                    .ledger
                    .execute(LedgerCommand::RenameFolder(
                        crate::core::ledger::command::RenameFolderPayload {
                            folder_id,
                            old_path: from_path,
                            new_path: to_path,
                        },
                    ))
                    .await;
            } else {
                // macOS new folder flow: "Pasta Sem Título" was never persisted,
                // so treat this as a brand-new folder discovery with the final name.
                info!(
                    "Indexer: Rename target '{}' not in DB — treating as new folder creation for '{}'",
                    from_path.display(),
                    to_path.display()
                );

                let to_path_str = to_path.to_string_lossy().to_string();
                // Don't duplicate if the target already exists
                if self.query_handler.find_folder_by_path(&to_path_str).await.unwrap_or(None).is_none() {
                    let parent_id = if let Some(parent) = to_path.parent() {
                        let parent_str = parent.to_string_lossy().to_string();
                        self.query_handler
                            .find_folder_by_path(&parent_str)
                            .await
                            .unwrap_or(None)
                    } else {
                        None
                    };

                    let folder_name = to_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let create_result = self
                        .ledger
                        .execute(LedgerCommand::CreateFolder(CreateFolderPayload {
                            parent_id: parent_id.clone(),
                            name: folder_name,
                            path: to_path.clone(),
                        }))
                        .await;

                    if let Ok(new_folder_asset) = create_result {
                        let _ = self.scan_directory(to_path, Some(new_folder_asset.id)).await;
                    } else {
                        let _ = self.scan_directory(to_path, parent_id).await;
                    }
                }
            }
        } else {
            let _ = self
                .ledger
                .execute(LedgerCommand::UpdateAsset(
                    crate::core::ledger::command::UpdateAssetPayload {
                        asset_id: None,
                        old_path: Some(from_path),
                        new_path: to_path,
                    },
                ))
                .await;
        }
    }

    /// Performs a differential repair of the library index.
    /// Finds assets with missing formats or thumbnails and updates them based on the registry.
    pub async fn repair_library(&self) -> AppResult<()> {
        info!("▶ Repair STARTED");

        let assets_needing_repair = self.query_handler.get_assets_needing_repair().await?;
        let count = assets_needing_repair.len();

        if count == 0 {
            info!("Repair COMPLETED: 0 assets needed repair.");
            return Ok(());
        }

        info!("Found {} assets needing repair.", count);

        for asset in assets_needing_repair {
            if asset.format_type == "unknown" {
                if let Some(supported_format) = self.registry.detect(&asset.path) {
                    let format_name = supported_format.name.to_string();
                    let _ = self
                        .ledger
                        .execute(LedgerCommand::UpdateFormat {
                            asset_id: asset.id.clone(),
                            format: format_name,
                        })
                        .await;
                }
            }

            if asset.thumbnail_path.is_none() {
                let _ = self
                    .ledger
                    .execute(LedgerCommand::RegenerateThumbnail {
                        asset_id: asset.id.clone(),
                    })
                    .await;
            }
        }

        info!("Repair COMPLETED: {} assets processed.", count);
        Ok(())
    }

    /// Ensures that the complete folder hierarchy for a given path exists in the database.
    /// Returns the ID of the leaf folder.
    #[async_recursion]
    async fn ensure_folder_hierarchy(&self, path: &std::path::Path) -> AppResult<Option<String>> {
        // 1. Check if it's already in the DB
        let path_str = path.to_string_lossy().to_string();
        if let Some(id) = self.query_handler.find_folder_by_path(&path_str).await? {
            return Ok(Some(id));
        }

        // 2. Resolve parent
        let parent_id = if let Some(parent) = path.parent() {
            // Check if parent exists, recursively
            self.ensure_folder_hierarchy(parent).await?
        } else {
            None
        };

        // 3. Create this folder (this will also trigger orphan adoption)
        let folder_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let create_folder_command = LedgerCommand::CreateFolder(CreateFolderPayload {
            parent_id,
            name: folder_name,
            path: path.to_path_buf(),
        });

        match self.ledger.execute(create_folder_command).await {
            Ok(folder_asset) => Ok(Some(folder_asset.id)),
            Err(e) => {
                warn!("Failed to ensure folder hierarchy for {:?}: {}", path, e);
                Ok(None)
            }
        }
    }

}

/// Pure classification function executed by each producer task.
/// Reads the filesystem metadata and compares with the comparison cache
/// to decide if a file needs indexing.
async fn classify_file_entry(
    entry: &DirEntry,
    comparison_cache: &HashMap<String, (i64, DateTime<Utc>)>,
    folder_cache: &Arc<RwLock<HashMap<PathBuf, String>>>,
    registry: &crate::core::formats::registry::FormatRegistry,
    root_folder_id: &Option<String>,
) -> AssetDiscoveryResult {
    let entry_path = entry.path().to_path_buf();
    let path_str = entry_path.to_string_lossy().to_string();

    // Read filesystem metadata
    let metadata = match entry.metadata() {
        Ok(metadata) => metadata,
        Err(error) => {
            return AssetDiscoveryResult::Error(format!(
                "Failed to read metadata for {:?}: {}",
                entry_path, error
            ));
        }
    };

    let disk_size = metadata.len() as i64;
    let disk_modified_time: DateTime<Utc> = metadata
        .modified()
        .ok()
        .map(|time| time.into())
        .unwrap_or_else(Utc::now);
    let disk_created_time: Option<DateTime<Utc>> = metadata.created().ok().map(|time| time.into());

    // Differential check: compare with cached data
    let needs_indexing =
        if let Some((cached_size, cached_modified_time)) = comparison_cache.get(&path_str) {
            disk_size != *cached_size
                || (disk_modified_time - *cached_modified_time)
                    .num_seconds()
                    .abs()
                    >= 1
        } else {
            true // New file — not in cache
        };

    if !needs_indexing {
        return AssetDiscoveryResult::ExistingAsset;
    }

    // Resolve format from registry
    let (format_name, family_name) = if let Some(supported_format) = registry.detect(&entry_path) {
        (
            supported_format.name.to_string(),
            supported_format.type_category.to_string(),
        )
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };

    // Resolve folder_id from cache
    let asset_folder_id = {
        let cache_read = folder_cache.read().await;
        entry_path
            .parent()
            .and_then(|parent| cache_read.get(parent).cloned())
            .or_else(|| root_folder_id.clone())
    };

    AssetDiscoveryResult::NewAsset(CreateAssetPayload {
        path: entry_path,
        file_size: disk_size as u64,
        format_type: format_name,
        family: family_name,
        state_init: AssetState::Indexed,
        folder_id: asset_folder_id,
        created_at: disk_created_time,
        modified_at: Some(disk_modified_time),
    })
}
