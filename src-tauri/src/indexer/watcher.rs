use super::types::{AddedItemContext, BatchChangePayload, RemovedItemContext, WatcherRegistry};
use crate::core::ledger::command::{CreateAssetPayload, LedgerCommand, UpdateAssetPayload};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::AssetState;
use crate::db::models::AssetMetadata;
use crate::db::Db;
use crate::indexer::metadata::get_asset_metadata;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn};

// ---------------------------------------------------------------------------
// Pipeline State structs
// ---------------------------------------------------------------------------

#[derive(Default)]
struct WatcherBuffer {
    added: HashMap<String, AssetMetadata>,
    added_folders: HashSet<String>,
    removed: HashSet<String>,
    renamed: HashMap<String, String>,
    pending_renames: HashMap<usize, String>,
    refresh_needed: bool,
}

impl WatcherBuffer {
    fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.added_folders.is_empty()
            && self.removed.is_empty()
            && self.renamed.is_empty()
            && !self.refresh_needed
    }
}

// ---------------------------------------------------------------------------
// Primary Watcher Process
// ---------------------------------------------------------------------------

/// Start a filesystem watcher for the given root path.
#[instrument(skip_all, fields(root = %root_str))]
pub fn start_watcher(
    app: AppHandle,
    db: Arc<Db>,
    registry: Arc<tokio::sync::Mutex<WatcherRegistry>>,
    ledger: Arc<dyn TransactionalAssetLedger>,
    path: PathBuf,
    root_str: String,
) -> JoinHandle<()> {
    let watch_path = path.canonicalize().unwrap_or(path);
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| PathBuf::from(""));
    let root_str_clone = root_str.clone();

    // Create a cancellation token for this watcher
    let watcher_token = CancellationToken::new();

    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<Event>(200);

        // Register token in registry (cancelling any previous watcher for this path)
        {
            let mut reg = registry.lock().await;
            if let Some(old_token) = reg
                .watchers
                .insert(root_str_clone.clone(), watcher_token.clone())
            {
                old_token.cancel();
            }
        }

        let mut watcher = match RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        ) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
            error!("Failed to watch path: {}", e);
            return;
        }
        let _watcher_ref = watcher; // Keep alive

        let mut buffer = WatcherBuffer::default();
        let mut timer = tokio::time::interval(Duration::from_millis(600));

        info!(
            "Watcher pipeline initialized for namespace: {}",
            root_str_clone
        );

        loop {
            tokio::select! {
                _ = watcher_token.cancelled() => {
                    info!("Watcher task received STOP signal");
                    break;
                }
                Some(event) = rx.recv() => {
                    // PHASE 1: Parse
                    if event.paths.iter().any(|p| p.starts_with(&app_data_dir)) { continue; }
                    phase_parse_and_normalize(&event, &root_str_clone, &mut buffer);
                }
                _ = timer.tick() => {
                    // PHASE 2 & 3: Classify & Heuristics
                    phase_classify_heuristics(&db, &mut buffer).await;

                    if buffer.is_empty() {
                        continue;
                    }

                    // PHASE 4: Persist
                    let (res_added, res_removed, res_updated, refresh) = phase_persist(&app, &db, &ledger, &mut buffer, &app_data_dir).await;

                    // PHASE 5: Emit
                    if !res_added.is_empty() || !res_removed.is_empty() || !res_updated.is_empty() || refresh {
                        phase_emit(&app, res_added, res_removed, res_updated, refresh);
                    }
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Pipeline Phases
// ---------------------------------------------------------------------------

/// Phase 1: Event Parsing and Data Normalization (Debouncing & deduplication)
#[instrument(skip_all)]
fn phase_parse_and_normalize(event: &Event, root_str: &str, buffer: &mut WatcherBuffer) {
    match event.kind {
        EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::Both)) => {
            if event.paths.len() == 2 {
                let from = normalize_path(&event.paths[0].to_string_lossy());
                let to = normalize_path(&event.paths[1].to_string_lossy());

                if buffer.added_folders.remove(&from) {
                    buffer.added_folders.insert(to);
                } else if let Some(meta) = buffer.added.remove(&from) {
                    buffer.added.insert(to, meta);
                } else {
                    buffer.renamed.insert(from, to);
                }
            }
        }
        EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::From)) => {
            if !event.paths.is_empty() {
                let path_str = normalize_path(&event.paths[0].to_string_lossy());
                if let Some(tracker) = event.attrs.tracker() {
                    buffer.pending_renames.insert(tracker, path_str);
                } else {
                    buffer.removed.insert(path_str);
                }
            }
        }
        EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::To)) => {
            if !event.paths.is_empty() {
                let path_str = normalize_path(&event.paths[0].to_string_lossy());
                let matched_from = match event.attrs.tracker() {
                    Some(tracker) => buffer.pending_renames.remove(&tracker),
                    None => None,
                };

                if let Some(from) = matched_from {
                    if buffer.added_folders.remove(&from) {
                        buffer.added_folders.insert(path_str.clone());
                    } else if let Some(meta) = buffer.added.remove(&from) {
                        buffer.added.insert(path_str.clone(), meta);
                    } else {
                        buffer.renamed.insert(from, path_str.clone());
                    }
                } else if path_str != root_str {
                    let path = &event.paths[0];
                    if path.is_dir() {
                        buffer.added_folders.insert(path_str);
                    } else if is_asset_file(path) {
                        if let Some(meta) = get_asset_metadata(path) {
                            buffer.added.insert(path_str, meta);
                        }
                    }
                }
            }
        }
        _ => {
            for path in &event.paths {
                let path_str = normalize_path(&path.to_string_lossy());
                if path.exists() {
                    if path_str != root_str {
                        if path.is_dir() {
                            buffer.removed.remove(&path_str);
                            buffer.added_folders.insert(path_str);
                        } else if is_asset_file(path) {
                            buffer.removed.remove(&path_str);
                            // Immediate Classification metadata extraction
                            if let Some(meta) = get_asset_metadata(path) {
                                buffer.added.insert(path_str, meta);
                            }
                        }
                    }
                } else {
                    buffer.added.remove(&path_str);
                    buffer.added_folders.remove(&path_str);
                    buffer.removed.insert(path_str);
                }
            }
        }
    }
}

/// Phase 2 & 3: Classification and Pair matching Heuristics
#[instrument(skip_all)]
async fn phase_classify_heuristics(db: &Arc<Db>, buffer: &mut WatcherBuffer) {
    for (_, path) in buffer.pending_renames.drain() {
        buffer.removed.insert(path);
    }

    // Heuristics for non-tracked renames
    let removed_list: Vec<String> = buffer.removed.iter().cloned().collect();
    for from_path in removed_list {
        if !buffer.removed.contains(&from_path) {
            continue;
        }

        let from_buf = Path::new(&from_path);

        // Folder Heuristic: Share parent
        let folder_match = buffer
            .added_folders
            .iter()
            .find(|to_path| Path::new(to_path).parent() == from_buf.parent())
            .cloned();

        if let Some(to_path) = folder_match {
            debug!("Pairing split FOLDER RENAME: {} -> {}", from_path, to_path);
            buffer.renamed.insert(from_path.clone(), to_path.clone());
            buffer.removed.remove(&from_path);
            buffer.added_folders.remove(&to_path);
            continue;
        }

        // Asset Heuristic: Metadata match using db constraints
        if is_asset_file(from_buf) {
            if let Ok(Some((size, created))) = db.get_file_comparison_data(&from_path).await {
                let asset_match = buffer
                    .added
                    .iter()
                    .find(|(_, m)| m.size == size && m.created_at == created)
                    .map(|(t, _)| t.clone());

                if let Some(to_path) = asset_match {
                    debug!("Pairing split ASSET RENAME: {} -> {}", from_path, to_path);
                    buffer.renamed.insert(from_path.clone(), to_path.clone());
                    buffer.removed.remove(&from_path);
                    buffer.added.remove(&to_path);
                }
            }
        }
    }
}

/// Phase 4: Atomic Persistence
/// Validates constraints and mutates SQLite respecting transactions where possible.
#[instrument(skip_all)]
async fn phase_persist(
    app: &AppHandle,
    db: &Arc<Db>,
    ledger: &Arc<dyn TransactionalAssetLedger>,
    buffer: &mut WatcherBuffer,
    app_data_dir: &Path,
) -> (
    Vec<AddedItemContext>,
    Vec<RemovedItemContext>,
    Vec<AddedItemContext>,
    bool,
) {
    let mut res_added = Vec::new();
    let mut res_removed = Vec::new();
    let mut res_updated = Vec::new();
    let mut actual_refresh_needed = buffer.refresh_needed;

    // A. Process Renames
    for (from, to) in buffer.renamed.drain() {
        let to_path = PathBuf::from(&to);
        let new_name = to_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if to_path.is_dir() {
            debug!("Processing FOLDER RENAME: {} -> {}", from, to);
            match db.rename_folder(&from, &to, &new_name).await {
                Ok(true) => debug!("Success folder rename: {} -> {}", from, to),
                Ok(false) => {
                    warn!(
                        "Folder rename returned false (source {} not in DB). Treating as New.",
                        from
                    );
                    buffer.added_folders.insert(to);
                }
                Err(e) => error!("Failed folder rename: {}", e),
            }
            actual_refresh_needed = true;
        } else {
            let parent = normalize_path(
                &to_path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            let folder_id = match db.get_folder_by_path(&parent).await {
                Ok(Some(id)) => id,
                _ => db.ensure_folder_hierarchy(&parent).await.unwrap_or(0),
            };

            if folder_id > 0 {
                match db.rename_asset(&from, &to, &new_name, folder_id).await {
                    Ok(Some((meta, old_fid))) => {
                        // V2 Ledger: Update
                        if let Err(e) = ledger
                            .execute(LedgerCommand::UpdateAsset(UpdateAssetPayload {
                                asset_id: None,
                                old_path: Some(PathBuf::from(&from)),
                                new_path: PathBuf::from(&to),
                            }))
                            .await
                        {
                            error!("Failed to record asset rename in V2 Ledger: {}", e);
                        }

                        res_updated.push(AddedItemContext {
                            metadata: meta,
                            folder_id,
                            old_folder_id: if old_fid != folder_id {
                                Some(old_fid)
                            } else {
                                None
                            },
                        });
                    }
                    _ => {
                        if let Some(meta) = get_asset_metadata(&to_path) {
                            buffer.added.insert(to, meta);
                        }
                    }
                }
            }
        }
    }

    // B. Process Removed
    for path in buffer.removed.drain() {
        let db_clone = db.clone();
        let app_clone = app.clone();
        let path_clone = path.clone();
        let data_dir_clone = app_data_dir.to_path_buf();

        // Immediate UI feedback for assets
        if let Ok(Some((img_id, fid, tags))) = db_clone.get_asset_context(&path).await {
            res_removed.push(RemovedItemContext {
                id: img_id,
                folder_id: fid,
                tag_ids: tags,
            });
        }

        let ledger_clone = ledger.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Before deleting, check if it's a folder or an asset
            if let Ok(Some((_img_id, _fid, _tags))) = db_clone.get_asset_context(&path_clone).await
            {
                // V2 Ledger: Delete
                if let Err(e) = ledger_clone
                    .execute(LedgerCommand::DeleteAsset {
                        asset_id: None,
                        path: Some(PathBuf::from(&path_clone)),
                        physical_delete: false,
                    })
                    .await
                {
                    error!("Failed to record asset deletion in V2 Ledger: {}", e);
                }

                if let Ok(Some(deleted_id)) = db_clone
                    .delete_asset_by_path_returning_context(&path_clone)
                    .await
                {
                    trace!("Finalized removal for: {}", path_clone);
                    let thumb = data_dir_clone
                        .join("thumbnails")
                        .join(format!("{}.webp", deleted_id.0)); // deleted_id is (id, old_fid, tags)
                    let _ = std::fs::remove_file(thumb);
                }
            } else if let Ok(Some(fid)) = db_clone.get_folder_by_path(&path_clone).await {
                if !std::path::Path::new(&path_clone).exists() {
                    trace!("Deleting folder (delay expired): {}", path_clone);
                    let _ = db_clone.delete_folder(fid).await;
                    let _ = app_clone.emit(
                        "library:batch-change",
                        BatchChangePayload {
                            added: vec![],
                            removed: vec![],
                            updated: vec![],
                            needs_refresh: true,
                        },
                    );
                }
            }
        });
    }

    // C. Process Added Folders
    for path in buffer.added_folders.drain() {
        trace!("Ensuring folder: {}", path);
        if db.ensure_folder_hierarchy(&path).await.is_ok() {
            actual_refresh_needed = true;
        }
    }

    // D. Process Added Assets
    for (path, meta) in buffer.added.drain() {
        let parent = normalize_path(
            &Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
        );
        if let Ok(fid) = db.ensure_folder_hierarchy(&parent).await {
            // Dual-write: Old DB
            match db.save_asset(fid, &meta).await {
                Ok((id, old_fid, is_new)) => {
                    let mut meta_with_id = meta.clone();
                    meta_with_id.id = id;

                    let ctx = AddedItemContext {
                        metadata: meta_with_id,
                        folder_id: fid,
                        old_folder_id: old_fid,
                    };

                    // V2 Ledger: Add
                    if is_new {
                        if let Err(e) = ledger
                            .execute(LedgerCommand::CreateAsset(CreateAssetPayload {
                                path: PathBuf::from(&meta.path),
                                file_size: meta.size as u64,
                                format_type: meta.format.clone(),
                                family: meta.media_type.clone(),
                                state_init: AssetState::Indexed,
                                folder_id: None,
                            }))
                            .await
                        {
                            error!("Failed to record asset add in V2 Ledger: {}", e);
                        }
                        res_added.push(ctx);
                    } else {
                        // TODO: LedgerCommand::UpdateAsset if needed
                        res_updated.push(ctx);
                    }
                }
                Err(e) => error!("Error saving {}: {}", path, e),
            }
        }
    }

    buffer.refresh_needed = false;
    (res_added, res_removed, res_updated, actual_refresh_needed)
}

/// Phase 5: Emit the payload to the frontend cluster safely
#[instrument(skip_all)]
fn phase_emit(
    app: &AppHandle,
    res_added: Vec<AddedItemContext>,
    res_removed: Vec<RemovedItemContext>,
    res_updated: Vec<AddedItemContext>,
    needs_refresh: bool,
) {
    if let Err(e) = app.emit(
        "library:batch-change",
        BatchChangePayload {
            added: res_added,
            removed: res_removed,
            updated: res_updated,
            needs_refresh,
        },
    ) {
        error!("Failed to emit batch-change to frontend: {}", e);
    } else {
        trace!("Batch emitted successfully");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize_path(path: &str) -> String {
    let p = path.trim_end_matches('/');
    if p.is_empty() {
        return "/".to_string();
    }
    p.to_string()
}

fn is_asset_file(path: &std::path::Path) -> bool {
    crate::formats::FileFormat::is_supported_extension(path)
}
