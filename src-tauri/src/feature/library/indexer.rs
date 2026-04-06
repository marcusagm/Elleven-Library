use crate::core::error::AppResult;
use crate::core::events::AppEventBus;
use crate::core::ledger::command::{CreateAssetPayload, CreateFolderPayload, LedgerCommand};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::AssetState;
use crate::core::repository::AssetQueryHandler;
use chrono::{DateTime, Utc};
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
        info!("▶ Scan STARTED for: {} (concurrency_limit={})", root_str, self.concurrency_limit);

        // Resolve folder_id if not provided
        let current_root_id = if let Some(id) = folder_id {
            Some(id)
        } else {
            self.query_handler.find_folder_by_path(&root_str).await?
        };

        // Emit ScanStarted
        let _ = self.event_bus.publish(crate::core::events::DomainEvent::ScanStarted {
            library_id: current_root_id.clone().unwrap_or_else(|| "root".to_string()),
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
            if let Some(existing_id) = self.query_handler.find_folder_by_path(&dir_path_str).await? {
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
        debug!("Loaded {} entries from comparison cache", comparison_cache.len());

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
                            .execute(LedgerCommand::BatchCreate(std::mem::take(&mut batch_payloads)))
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
            if processed_count % 100 == 0 || processed_count == total_files as u64 {
                let _ = self.event_bus.publish(crate::core::events::DomainEvent::ScanProgress {
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
                if let Err(e) = self.ledger.execute(LedgerCommand::DeleteAsset {
                    asset_id: None,
                    path: Some(PathBuf::from(cached_path)),
                    physical_delete: false,
                }).await {
                    warn!("Failed to prune stale file {}: {}", cached_path, e);
                } else {
                    pruned_files_count += 1;
                }
            }
        }

        // Prune Missing Folders
        for folder in existing_folders {
            let folder_path_str = folder.path.to_string_lossy().to_string();
            
            if folder_path_str.starts_with(&root_str) && folder_path_str != root_str && !verified_folder_paths.contains(&folder_path_str) {
                if let Err(e) = self.ledger.execute(LedgerCommand::RemoveFolder(
                    crate::core::ledger::command::RemoveFolderPayload { folder_id: folder.id.clone() }
                )).await {
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
        let _ = self.event_bus.publish(crate::core::events::DomainEvent::ScanCompleted {
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
            // ─── New File Detected ──────────────────────────────────────
            DomainEvent::FsFileDiscovered {
                path,
                size_bytes,
            } => {
                let entry_path = PathBuf::from(&path);

                // Skip unsupported formats
                let is_supported = entry_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(|extension| self.registry.is_supported_extension(extension))
                    .unwrap_or(false);

                if !is_supported {
                    debug!("EventListener: skipping unsupported file: {}", path);
                    return;
                }

                // Resolve format from registry
                let (format_name, family_name) =
                    if let Some(supported_format) = self.registry.detect(&entry_path) {
                        (
                            supported_format.name.to_string(),
                            supported_format.type_category.to_string(),
                        )
                    } else {
                        ("unknown".to_string(), "unknown".to_string())
                    };

                // Read filesystem timestamps
                let (created_at, modified_at) = match std::fs::metadata(&entry_path) {
                    Ok(metadata) => (
                        metadata.created().ok().map(|time| {
                            let datetime: chrono::DateTime<chrono::Utc> = time.into();
                            datetime
                        }),
                        metadata.modified().ok().map(|time| {
                            let datetime: chrono::DateTime<chrono::Utc> = time.into();
                            datetime
                        }),
                    ),
                    Err(_) => (None, None),
                };

                // Resolve folder_id from parent path
                let folder_id = if let Some(parent) = entry_path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    match self.query_handler.find_folder_by_path(&parent_str).await {
                        Ok(folder_id_option) => folder_id_option,
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let create_payload = CreateAssetPayload {
                    path: entry_path,
                    file_size: size_bytes,
                    format_type: format_name,
                    family: family_name,
                    state_init: crate::core::models::asset::AssetState::Indexed,
                    folder_id,
                    created_at,
                    modified_at,
                };

                match self
                    .ledger
                    .execute(LedgerCommand::CreateAsset(create_payload))
                    .await
                {
                    Ok(asset) => {
                        info!(
                            "EventListener: created asset {} for: {}",
                            asset.id, path
                        );
                    }
                    Err(error) => {
                        // Duplicate path is expected if file already indexed — not an error
                        debug!(
                            "EventListener: create failed for {} (may already exist): {}",
                            path, error
                        );
                    }
                }
            }

            // ─── File/Asset Deleted ─────────────────────────────────────
            DomainEvent::FsPathDeleted { path } => {
                let entry_path = PathBuf::from(&path);

                match self
                    .ledger
                    .execute(LedgerCommand::DeleteAsset {
                        asset_id: None,
                        path: Some(entry_path),
                        physical_delete: false,
                    })
                    .await
                {
                    Ok(_) => {
                        info!("EventListener: deleted asset at: {}", path);
                    }
                    Err(error) => {
                        debug!(
                            "EventListener: delete failed for {} (may be folder or not indexed): {}",
                            path, error
                        );
                    }
                }
            }

            // ─── File/Folder Renamed or Moved ───────────────────────────
            DomainEvent::FsPathRenamed { from, to } => {
                let from_path = PathBuf::from(&from);
                let to_path = PathBuf::from(&to);

                if to_path.is_dir() {
                    // Directory rename — update all assets under this subtree
                    // by updating the folder path and re-pathing child assets
                    let from_str = from_path.to_string_lossy().to_string();

                    // Update the folder entry itself
                    if let Ok(Some(folder_id)) =
                        self.query_handler.find_folder_by_path(&from_str).await
                    {
                        let new_name = to_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        // Update folder name and path via a direct query approach
                        // For now, emit a log — the full folder rename cascading
                        // is handled by the scan_directory on next boot
                        info!(
                            "EventListener: folder renamed {} → {} (id: {}), name: {}",
                            from, to, folder_id, new_name
                        );
                    }
                } else {
                    // Single file rename/move
                    let update_payload = crate::core::ledger::command::UpdateAssetPayload {
                        asset_id: None,
                        old_path: Some(from_path),
                        new_path: to_path,
                    };

                    match self
                        .ledger
                        .execute(LedgerCommand::UpdateAsset(update_payload))
                        .await
                    {
                        Ok(asset) => {
                            info!(
                                "EventListener: renamed asset {} from {} → {}",
                                asset.id, from, to
                            );
                        }
                        Err(error) => {
                            warn!(
                                "EventListener: rename failed {} → {}: {}",
                                from, to, error
                            );
                        }
                    }
                }
            }

            // ─── New Directory Detected ─────────────────────────────────
            DomainEvent::FsDirectoryDiscovered { path } => {
                let dir_path = PathBuf::from(&path);
                let dir_path_str = dir_path.to_string_lossy().to_string();

                // Check if folder already exists
                if let Ok(Some(_)) = self
                    .query_handler
                    .find_folder_by_path(&dir_path_str)
                    .await
                {
                    return; // Already exists
                }

                // Resolve parent folder
                let parent_id = if let Some(parent) = dir_path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    match self.query_handler.find_folder_by_path(&parent_str).await {
                        Ok(folder_id_option) => folder_id_option,
                        Err(_) => None,
                    }
                } else {
                    None
                };

                let folder_name = dir_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                match self
                    .ledger
                    .execute(LedgerCommand::CreateFolder(CreateFolderPayload {
                        parent_id,
                        name: folder_name,
                        path: dir_path,
                    }))
                    .await
                {
                    Ok(_) => {
                        info!("EventListener: created folder: {}", path);
                    }
                    Err(error) => {
                        warn!("EventListener: create folder failed for {}: {}", path, error);
                    }
                }
            }

            // ─── Directory Deleted ──────────────────────────────────────
            DomainEvent::FsDirectoryDeleted { path } => {
                let dir_path_str = path.clone();

                match self
                    .query_handler
                    .find_folder_by_path(&dir_path_str)
                    .await
                {
                    Ok(Some(folder_id)) => {
                        match self
                            .ledger
                            .execute(LedgerCommand::RemoveFolder(
                                crate::core::ledger::command::RemoveFolderPayload {
                                    folder_id: folder_id.clone(),
                                },
                            ))
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "EventListener: removed folder {} (id: {})",
                                    path, folder_id
                                );
                            }
                            Err(error) => {
                                warn!(
                                    "EventListener: remove folder failed for {}: {}",
                                    path, error
                                );
                            }
                        }
                    }
                    _ => {
                        debug!(
                            "EventListener: folder not found in DB for deletion: {}",
                            path
                        );
                    }
                }
            }

            // ─── Ignore all other events ────────────────────────────────
            _ => {}
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
    let needs_indexing = if let Some((cached_size, cached_modified_time)) = comparison_cache.get(&path_str) {
        disk_size != *cached_size || (disk_modified_time - *cached_modified_time).num_seconds().abs() >= 1
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
