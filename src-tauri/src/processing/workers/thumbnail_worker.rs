use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};

use crate::core::error::{AppError, AppResult};
use crate::core::formats::FormatRegistry;
use crate::core::ledger::command::LedgerCommand;
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::repository::AssetQueryHandler;
use crate::core::workflows::thumbnails::priority::ThumbnailPriorityState;

/// Background worker that orchestrates thumbnail generation.
///
/// Under Hexagonal Architecture, this is a "Processing Worker" (Infrastructure/Application layer)
/// that implements the workflow of consuming assets and producing thumbnails.
/// It uses a Hybrid Queue:
/// 1. LIFO: High-priority items from `ThumbnailPriorityState` (UI-driven).
/// 2. FIFO: Background items from the Database (Indexing-driven).
pub struct ThumbnailWorker {
    /// Registry for resolving format-specific thumbnail capabilities.
    format_registry: Arc<FormatRegistry>,
    /// Ledger for committing state changes (UpdateThumbnail).
    ledger: Arc<dyn TransactionalAssetLedger>,
    /// Query handler for fetching assets waiting for thumbnails.
    query_handler: Arc<dyn AssetQueryHandler>,
    /// Shared state for UI-requested priorities.
    priority_state: Arc<ThumbnailPriorityState>,
    /// Output directory for thumbnails.
    thumbnails_dir: PathBuf,
    /// Number of concurrent extraction workers.
    worker_threads: usize,
}

/// Implementation of the ThumbnailWorker struct.
impl ThumbnailWorker {
    /// Creates a new instance of the ThumbnailWorker.
    ///
    /// # Arguments
    ///
    /// * `format_registry` - The registry for resolving format-specific thumbnail capabilities.
    /// * `ledger` - The ledger for committing state changes (UpdateThumbnail).
    /// * `query_handler` - The query handler for fetching assets waiting for thumbnails.
    /// * `priority_state` - The shared state for UI-requested priorities.
    /// * `thumbnails_dir` - The output directory for thumbnails.
    /// * `worker_threads` - The number of concurrent extraction workers.
    ///
    /// # Returns
    ///
    /// A new instance of `ThumbnailWorker`.
    pub fn new(
        format_registry: Arc<FormatRegistry>,
        ledger: Arc<dyn TransactionalAssetLedger>,
        query_handler: Arc<dyn AssetQueryHandler>,
        priority_state: Arc<ThumbnailPriorityState>,
        thumbnails_dir: PathBuf,
        worker_threads: usize,
    ) -> Self {
        Self {
            format_registry,
            ledger,
            query_handler,
            priority_state,
            thumbnails_dir,
            worker_threads,
        }
    }

    /// Starts the orchestration loop in a background task.
    ///
    /// # Arguments
    ///
    /// * `token` - The cancellation token for the background task.
    ///
    /// # Returns
    ///
    /// A `JoinHandle` for the background task.
    pub fn start(self, token: CancellationToken) -> tauri::async_runtime::JoinHandle<()> {
        tauri::async_runtime::spawn(async move {
            info!("ThumbnailWorker: Orchestrator loop started");
            let worker_arc = Arc::new(self);

            // Create a dedicated rayon thread pool for CPU-bound work
            let pool = match rayon::ThreadPoolBuilder::new()
                .num_threads(worker_arc.worker_threads)
                .thread_name(|index| format!("thumb-worker-{}", index))
                .build()
            {
                Ok(p) => Arc::new(p),
                Err(e) => {
                    error!("ThumbnailWorker: Failed to create thread pool: {}", e);
                    return;
                }
            };

            loop {
                if token.is_cancelled() {
                    info!("ThumbnailWorker: Shutdown requested");
                    break;
                }

                match worker_arc.process_batch(&pool).await {
                    Ok(processed_count) => {
                        if processed_count == 0 {
                            // Empty queues: wait before next poll
                            tokio::select! {
                                _ = token.cancelled() => break,
                                _ = sleep(Duration::from_millis(500)) => continue,
                            }
                        }
                    }
                    Err(e) => {
                        error!("ThumbnailWorker: Batch process error: {}", e);
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
        })
    }

    /// Processes a single batch of work, prioritizing LIFO over FIFO.
    ///
    /// # Arguments
    ///
    /// * `pool` - The thread pool for parallel processing.
    ///
    /// # Returns
    ///
    /// A `Result` containing the number of processed assets or an error.
    #[instrument(skip_all)]
    async fn process_batch(&self, pool: &Arc<rayon::ThreadPool>) -> AppResult<usize> {
        // 1. Fetch Priority Items (LIFO)
        let mut asset_ids = self.priority_state.pop_batch(10);
        let is_priority = !asset_ids.is_empty();

        // 2. Fallback to Background Items (FIFO)
        if asset_ids.is_empty() {
            // Note: The repository needs a way to query "missing thumbnails".
            // For now, we assume the query_handler can handle an internal query or we add a specific method.
            // Requirement from sprint: SELECT id FROM assets WHERE thumbnail_path IS NULL LIMIT 10
            asset_ids = self.query_handler.get_assets_needing_thumbnails(10).await?;
        }

        if asset_ids.is_empty() {
            return Ok(0);
        }

        debug!(
            "ThumbnailWorker: Processing batch of {} (priority: {})",
            asset_ids.len(),
            is_priority
        );

        // 3. Resolve Assets
        let mut tasks = Vec::new();
        for id in asset_ids {
            if let Ok(asset) = self.query_handler.get_asset_by_id(&id).await {
                if let Some(format_provider) = self.format_registry.resolve(&asset.path, &[]) {
                    if format_provider.thumbnail().is_some() {
                        tasks.push((asset, format_provider));
                    }
                }
            }
        }

        let processed_count = tasks.len();

        // 4. Parallel Generation
        let thumbnails_dir = self.thumbnails_dir.clone();
        let pool_clone = pool.clone();
        
        // Capture the Tokio runtime handle to use inside Rayon threads
        let handle = tokio::runtime::Handle::current();

        let results: Vec<(String, AppResult<Vec<u8>>, Option<AppResult<(Vec<u8>, String)>>)> = pool_clone.install(|| {
            use rayon::prelude::*;
            tasks
                .into_par_iter()
                .map(|(asset, provider)| {
                    let id = asset.id.clone();
                    let (thumb_res, preview_res) = handle.block_on(async {
                        let t = if let Some(capability) = provider.thumbnail() {
                            capability.generate(&asset.path, &id, 300).await
                        } else {
                            Err(AppError::Internal("Thumbnail capability not found".to_string()))
                        };

                        let p = if let Some(capability) = provider.preview() {
                            Some(capability.generate_preview(&asset.path, &id).await)
                        } else {
                            None
                        };

                        (t, p)
                    });
                    (id, thumb_res, preview_res)
                })
                .collect()
        });

        // 5. Commit Results
        for (id, thumb_result, preview_result) in results {
            // 5a. Handle Preview (e.g. converted GLB for 3D)
            if let Some(Ok((preview_bytes, mime))) = preview_result {
                let extension = match mime.as_str() {
                    "model/gltf-binary" => "glb",
                    "model/gltf+json" => "gltf",
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    _ => "bin",
                };
                let preview_filename = format!("{}.{}", id, extension);
                let preview_path = thumbnails_dir.join(&preview_filename);
                
                if let Err(e) = std::fs::write(&preview_path, preview_bytes) {
                    error!("ThumbnailWorker: Failed to write preview for {}: {}", id, e);
                }
            }

            // 5b. Handle Thumbnail
            match thumb_result {
                Ok(bytes) => {
                    // Generate unique filename: {hash_of_id}.webp
                    let filename = format!("{}.webp", id);
                    let output_path = thumbnails_dir.join(&filename);

                    // 5c. Transcode to valid WebP to ensure consistency and fix "Invalid Chunk header"
                    let transcode_result = pool_clone.install(|| {
                        let img = image::load_from_memory(&bytes).map_err(|e| {
                            AppError::Internal(format!("Failed to load thumbnail for transcoding: {}", e))
                        })?;
                        
                        let encoder = webp::Encoder::from_image(&img).map_err(|e| {
                            AppError::Internal(format!("Failed to create WebP encoder: {}", e))
                        })?;
                        
                        let webp_data = encoder.encode(75.0);
                        Ok::<Vec<u8>, AppError>(webp_data.to_vec())
                    });

                    let final_bytes = match transcode_result {
                        Ok(b) => b,
                        Err(e) => {
                            error!("ThumbnailWorker: Transcoding failed for {}: {}", id, e);
                            continue;
                        }
                    };

                    // Save to disk
                    if let Err(e) = std::fs::write(&output_path, final_bytes) {
                        error!(
                            "ThumbnailWorker: Failed to write thumbnail for {}: {}",
                            id, e
                        );
                        continue;
                    }

                    // Commit to Ledger
                    let command = LedgerCommand::UpdateThumbnail {
                        asset_id: id.clone(),
                        thumbnail_path: filename,
                    };

                    if let Err(e) = self.ledger.execute(command).await {
                        error!("ThumbnailWorker: Ledger commit failed for {}: {}", id, e);
                    }
                }
                Err(e) => {
                    error!("ThumbnailWorker: Generation failed for {}: {}", id, e);
                }
            }
        }

        Ok(processed_count)
    }
}
