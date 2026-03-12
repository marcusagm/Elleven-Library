use crate::core::error::AppResult;
use crate::core::events::AppEventBus;
use crate::core::ledger::command::{CreateAssetPayload, LedgerCommand};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::AssetState;
use crate::core::repository::AssetQueryHandler;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use walkdir::WalkDir;

/// Service responsible for indexing library folders.
///
/// Ref: Sprint 4.2 - "Diferencial de Estado (Scan Initial)"
pub struct LibraryIndexer {
    /// Port for read-only asset operations (differential cache).
    query_handler: Arc<dyn AssetQueryHandler>,
    /// Port for state mutations.
    ledger: Arc<dyn TransactionalAssetLedger>,
    /// Event bus for publishing domain events.
    event_bus: Arc<dyn AppEventBus>,
}

/// Implementation of the LibraryIndexer struct.
impl LibraryIndexer {
    /// Create a new LibraryIndexer.
    ///
    /// # Arguments
    ///
    /// * `query_handler` - Port for read-only asset operations (differential cache).
    /// * `ledger` - Port for state mutations.
    ///
    /// # Returns
    ///
    /// * `Self` - A new LibraryIndexer instance.
    pub fn new(
        query_handler: Arc<dyn AssetQueryHandler>,
        ledger: Arc<dyn TransactionalAssetLedger>,
        event_bus: Arc<dyn AppEventBus>,
    ) -> Self {
        Self {
            query_handler,
            ledger,
            event_bus,
        }
    }

    /// Perform a differential scan of a directory tree.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory to scan.
    ///
    /// # Returns
    ///
    /// * `AppResult<()>` - Result of the scan operation.
    #[instrument(skip(self), fields(path = %path.display()))]
    pub async fn scan_directory(&self, path: PathBuf, folder_id: Option<String>) -> AppResult<()> {
        let root_str = path.to_string_lossy().to_string();
        info!("Starting differential scan for: {}", root_str);

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

        // 1. Initial walk to count total files for progress reporting
        let total_files = WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && crate::formats::FileFormat::is_supported_extension(e.path()))
            .count();

        debug!("Total files to process: {}", total_files);

        // 2. Load comparison cache (Size, MTime)
        let cache = self
            .query_handler
            .get_all_files_comparison_data(&root_str)
            .await?;
        debug!("Loaded {} items from cache", cache.len());

        let mut discovered_count = 0;
        let mut processed_count = 0;
        let mut unchanged_count = 0;

        // 3. Iterate filesystem
        let mut folder_cache = std::collections::HashMap::new();
        if let Some(id) = current_root_id.clone() {
            folder_cache.insert(path.clone(), id);
        }

        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path().to_path_buf();
            
            if entry.file_type().is_dir() {
                // Ensure folder hierarchy exists in DB
                if entry_path == path {
                    continue; // Root already handled
                }

                let path_str = entry_path.to_string_lossy().to_string();
                if let Some(existing_id) = self.query_handler.find_folder_by_path(&path_str).await? {
                    folder_cache.insert(entry_path, existing_id);
                } else {
                    // Create folder
                    let parent_path = entry_path.parent().unwrap_or(&path);
                    let parent_id = folder_cache.get(parent_path).cloned();
                    
                    let name = entry_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let cmd = LedgerCommand::CreateFolder(crate::core::ledger::command::CreateFolderPayload {
                        parent_id,
                        name,
                        path: entry_path.clone(),
                    });

                    if let Ok(folder_asset) = self.ledger.execute(cmd).await {
                        folder_cache.insert(entry_path, folder_asset.id);
                    }
                }
                continue;
            }

            let path_str = entry_path.to_string_lossy().to_string();

            // Filter supported assets
            if !crate::formats::FileFormat::is_supported_extension(&entry_path) {
                continue;
            }

            processed_count += 1;
            
            // Emit progress
            if processed_count % 10 == 0 || processed_count == total_files {
                let _ = self.event_bus.publish(crate::core::events::DomainEvent::ScanProgress {
                    total: total_files,
                    processed: processed_count,
                    current_file: entry_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                });
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to read metadata for {:?}: {}", entry_path, e);
                    continue;
                }
            };

            let disk_size = metadata.len() as i64;
            let disk_mtime: DateTime<Utc> = metadata
                .modified()
                .ok()
                .map(|t| t.into())
                .unwrap_or_else(Utc::now);

            // 4. Differential Check
            let needs_index = if let Some((db_size, db_mtime)) = cache.get(&path_str) {
                disk_size != *db_size || (disk_mtime - *db_mtime).num_seconds().abs() >= 1
            } else {
                true // New file
            };

            if needs_index {
                debug!("Indexing asset: {}", path_str);

                let asset_folder_id = entry_path.parent()
                    .and_then(|p| folder_cache.get(p))
                    .cloned()
                    .or(current_root_id.clone());

                let cmd = LedgerCommand::CreateAsset(CreateAssetPayload {
                    path: entry_path.clone(),
                    file_size: disk_size as u64,
                    format_type: "unknown".to_string(),
                    family: "unknown".to_string(),
                    state_init: AssetState::Indexed,
                    folder_id: asset_folder_id,
                });

                if let Err(e) = self.ledger.execute(cmd).await {
                    error!("Failed to index {}: {}", path_str, e);
                } else {
                    discovered_count += 1;
                }
            } else {
                unchanged_count += 1;
            }
        }

        info!(
            "Scan complete. Discovered/Updated: {}, Unchanged: {}",
            discovered_count, unchanged_count
        );

        // Emit ScanCompleted
        let _ = self.event_bus.publish(crate::core::events::DomainEvent::ScanCompleted {
            library_id: current_root_id.unwrap_or_else(|| "root".to_string()),
        });

        Ok(())
    }

    /// Starts a background loop that listens for domain events and triggers scans or updates.
    ///
    /// # Arguments
    ///
    /// * `self` - The LibraryIndexer instance.
    /// * `event_bus` - Broadcast bus for publishing domain events.
    pub async fn start_event_listener(self: Arc<Self>, event_bus: Arc<dyn AppEventBus>) {
        let mut rx = event_bus.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    crate::core::events::DomainEvent::FsFileDiscovered { path, .. } => {
                        let path_buf = PathBuf::from(path);
                        if let Err(e) = self.scan_directory(path_buf, None).await {
                            error!("Background scan failed: {}", e);
                        }
                    }
                    crate::core::events::DomainEvent::FsPathRenamed { from: _, to } => {
                        let path_buf = PathBuf::from(to);
                        if let Err(e) = self.scan_directory(path_buf, None).await {
                            error!("Background scan (after rename) failed: {}", e);
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
