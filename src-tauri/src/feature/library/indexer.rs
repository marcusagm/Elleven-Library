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
    ) -> Self {
        Self {
            query_handler,
            ledger,
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
        let resolved_folder_id = if let Some(id) = folder_id {
            Some(id)
        } else {
            self.query_handler.find_folder_by_path(&root_str).await?
        };

        if resolved_folder_id.is_none() {
            debug!("Parent folder not found in database for path: {}. Assets will be untagged/root-level.", root_str);
        }

        // 1. Load comparison cache (Size, MTime)
        let cache = self
            .query_handler
            .get_all_files_comparison_data(&root_str)
            .await?;
        debug!("Loaded {} items from cache", cache.len());

        let mut discovered_count = 0;
        let mut unchanged_count = 0;

        // 2. Iterate filesystem
        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                continue;
            }

            let path_buf = entry.path().to_path_buf();
            let path_str = path_buf.to_string_lossy().to_string();

            // Filter supported assets (using legacy registry for now, or just extensions)
            if !crate::formats::FileFormat::is_supported_extension(&path_buf) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to read metadata for {:?}: {}", path_buf, e);
                    continue;
                }
            };

            let disk_size = metadata.len() as i64;
            let disk_mtime: DateTime<Utc> = metadata
                .modified()
                .ok()
                .map(|t| t.into())
                .unwrap_or_else(Utc::now);

            // 3. Differential Check
            let needs_index = if let Some((db_size, db_mtime)) = cache.get(&path_str) {
                // Strict comparison: size must match and time difference < 1s
                disk_size != *db_size || (disk_mtime - *db_mtime).num_seconds().abs() >= 1
            } else {
                true // New file
            };

            if needs_index {
                debug!("Indexing asset: {}", path_str);

                // Prepare Command for Ledger
                // We'll use "Indexed" or "Discovered" as initial state.
                // The Ledger will handle the atomic DB write.
                let cmd = LedgerCommand::CreateAsset(CreateAssetPayload {
                    path: path_buf,
                    file_size: disk_size as u64,
                    format_type: "unknown".to_string(), // FormatRegistry will refine this later
                    family: "unknown".to_string(),
                    state_init: AssetState::Indexed,
                    folder_id: resolved_folder_id.clone(),
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
