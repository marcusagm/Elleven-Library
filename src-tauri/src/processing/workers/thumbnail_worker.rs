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
    ) -> Self {
        Self {
            format_registry,
            ledger,
            query_handler,
            priority_state,
            thumbnails_dir,
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

            loop {
                if token.is_cancelled() {
                    info!("ThumbnailWorker: Shutdown requested");
                    break;
                }

                match worker_arc.process_batch().await {
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
    /// # Returns
    ///
    /// A `Result` containing the number of processed assets or an error.
    #[instrument(skip_all)]
    async fn process_batch(&self) -> AppResult<usize> {
        // 1. Fetch Priority Items (LIFO)
        let mut asset_ids = self.priority_state.pop_batch(10);
        let is_priority = !asset_ids.is_empty();

        // 2. Fallback to Background Items (FIFO)
        if asset_ids.is_empty() {
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
                    tasks.push((asset, format_provider));
                }
            }
        }

        let processed_count = tasks.len();

        // 4. Parallel Generation & Extraction
        let mut join_set = tokio::task::JoinSet::new();

        for (asset, provider) in tasks {
            let id = asset.id.clone();
            let path = asset.path.clone();
            let provider = provider.clone();
            let ledger = self.ledger.clone();

            join_set.spawn(async move {
                // 4a. Technical Metadata Extraction
                if let Some(meta_cap) = provider.metadata() {
                    if let Ok(tech_meta) = meta_cap.extract_technical(&path).await {
                        let width = tech_meta.get("width").and_then(|v| v.as_i64());
                        let height = tech_meta.get("height").and_then(|v| v.as_i64());
                        let duration = tech_meta.get("duration").and_then(|v| v.as_f64());

                        let _ = ledger
                            .execute(LedgerCommand::UpdateTechnicalMetadata(
                                crate::core::ledger::command::UpdateTechnicalMetadataPayload {
                                    asset_id: id.clone(),
                                    width,
                                    height,
                                    duration_secs: duration,
                                    technical_payload: Some(tech_meta),
                                    semantic_payload: None,
                                },
                            ))
                            .await;
                    }
                }

                // 4b. Thumbnail & Preview Generation
                let thumb_res = if let Some(thumb_cap) = provider.thumbnail() {
                    thumb_cap.generate(&path, &id, 300).await
                } else {
                    Err(AppError::Internal("Format does not support thumbnails".to_string()))
                };

                let preview_res = if let Some(preview_cap) = provider.preview() {
                    Some(preview_cap.generate_preview(&path, &id).await)
                } else {
                    None
                };

                (id, thumb_res, preview_res)
            });
        }

        // 5. Commit Results
        let thumbnails_dir = self.thumbnails_dir.clone();
        while let Some(join_res) = join_set.join_next().await {
            let (id, thumb_res, preview_res) = match join_res {
                Ok(r) => r,
                Err(e) => {
                    error!("ThumbnailWorker: Task panicked: {}", e);
                    continue;
                }
            };

            // 5a. Save Preview
            if let Some(Ok((preview_bytes, mime))) = preview_res {
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

            // 5b. Handle Thumbnail result
            match thumb_res {
                Ok(bytes) => {
                    let id_for_io = id.clone();
                    // CPU-Bound Transcoding to consistent WebP
                    let final_bytes_res = tokio::task::spawn_blocking(move || {
                        let img = image::load_from_memory(&bytes).map_err(|e| {
                            AppError::Internal(format!(
                                "Failed to load thumbnail for {}: {}",
                                id_for_io, e
                            ))
                        })?;

                        let encoder = webp::Encoder::from_image(&img).map_err(|e| {
                            AppError::Internal(format!("Failed to create WebP encoder: {}", e))
                        })?;

                        Ok::<Vec<u8>, AppError>(encoder.encode(75.0).to_vec())
                    })
                    .await;

                    let final_bytes = match final_bytes_res {
                        Ok(Ok(b)) => b,
                        Ok(Err(e)) => {
                            error!("ThumbnailWorker: Transcoding failed for {}: {}", id, e);
                            continue;
                        }
                        Err(e) => {
                            error!("ThumbnailWorker: Transcoding task joined with error for {}: {}", id, e);
                            continue;
                        }
                    };

                    let filename = format!("{}.webp", id);
                    let output_path = thumbnails_dir.join(&filename);

                    if let Err(e) = std::fs::write(&output_path, final_bytes) {
                        error!(
                            "ThumbnailWorker: Failed to write thumbnail for {}: {}",
                            id, e
                        );
                        continue;
                    }

                    // Force commit entry to database
                    if let Err(e) = self
                        .ledger
                        .execute(LedgerCommand::UpdateThumbnail {
                            asset_id: id.clone(),
                            thumbnail_path: filename.clone(),
                        })
                        .await
                    {
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
