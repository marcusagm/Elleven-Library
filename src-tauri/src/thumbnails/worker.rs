use crate::db::models::AssetColor;
use crate::db::Db;
use crate::thumbnails::priority::ThumbnailPriorityState;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Background worker that continuously generates thumbnails for indexed assets.
///
/// Processes a priority queue first (user-visible items), then falls back to
/// the regular queue of assets awaiting thumbnails. The worker cooperatively
/// shuts down when its `CancellationToken` is cancelled, finishing any
/// in-progress batch before exiting.
pub struct ThumbnailWorker {
    /// Shared database connection.
    db: Arc<Db>,
    /// Directory where generated thumbnails are stored.
    thumbnails_dir: PathBuf,
    /// Handle to the Tauri application for emitting events.
    app_handle: AppHandle,
    /// Application configuration snapshot.
    config: crate::settings::config::AppConfig,
    /// Shared priority state for on-demand thumbnail generation.
    priority_state: Arc<ThumbnailPriorityState>,
}

impl ThumbnailWorker {
    /// Create a new thumbnail worker instance.
    pub fn new(
        db: Arc<Db>,
        thumbnails_dir: PathBuf,
        app_handle: AppHandle,
        config: crate::settings::config::AppConfig,
        priority_state: Arc<ThumbnailPriorityState>,
    ) -> Self {
        Self {
            db,
            thumbnails_dir,
            app_handle,
            config,
            priority_state,
        }
    }

    /// Start the thumbnail worker loop on a background task.
    ///
    /// Returns the `JoinHandle` so callers can register it in the
    /// `LifecycleRegistry` for graceful shutdown tracking.
    ///
    /// The worker checks its `CancellationToken` between batches. When
    /// cancelled, it finishes the current in-progress batch (so that
    /// thumbnails being generated are not lost) and then exits. Any
    /// remaining items stay in the DB queue with `thumbnail_path IS NULL`
    /// and will be picked up on the next application start.
    pub fn start(self, token: CancellationToken) -> JoinHandle<()> {
        let db = self.db.clone();
        let app = self.app_handle.clone();
        let thumb_dir = self.thumbnails_dir.clone();
        let config = self.config.clone();
        let priority_state = self.priority_state.clone();

        let light_q = Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::new()));
        let heavy_q = Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::new()));
        let notify_light = Arc::new(tokio::sync::Notify::new());
        let notify_heavy = Arc::new(tokio::sync::Notify::new());

        // --- 1. LIGHT THREAD POOL WORKER ---
        let db_light = db.clone();
        let app_light = app.clone();
        let thumb_dir_light = thumb_dir.clone();
        let token_light = token.clone();
        let num_threads = config.thumbnail_threads;
        let light_queue = light_q.clone();
        let notify_light_queue = notify_light.clone();

        tauri::async_runtime::spawn(async move {
            Self::worker_loop(
                light_queue,
                notify_light_queue,
                db_light,
                app_light,
                thumb_dir_light,
                token_light,
                num_threads,
                20,
            )
            .await;
        });

        // --- 2. HEAVY (FFMPEG) THREAD POOL WORKER ---
        let db_heavy = db.clone();
        let app_heavy = app.clone();
        let thumb_dir_heavy = thumb_dir.clone();
        let token_heavy = token.clone();
        let heavy_threads = 2; // FFMPEG requires strict concurrency to avoid CPU exhaustion
        let heavy_queue = heavy_q.clone();
        let notify_heavy_queue = notify_heavy.clone();

        tauri::async_runtime::spawn(async move {
            Self::worker_loop(
                heavy_queue,
                notify_heavy_queue,
                db_heavy,
                app_heavy,
                thumb_dir_heavy,
                token_heavy,
                heavy_threads,
                4,
            )
            .await;
        });

        // --- 3. DISPATCHER LOOP ---
        tauri::async_runtime::spawn(async move {
            loop {
                if token.is_cancelled() {
                    info!("LIFECYCLE: ThumbnailWorker Dispatcher shutting down");
                    break;
                }

                let mut did_work = false;

                // 1. Check Priority Queue First (LIFO behavior per block)
                let priority_ids = priority_state.take_batch(config.indexer_batch_size as usize);
                if !priority_ids.is_empty() {
                    if let Ok(priority_imgs) =
                        db.get_assets_needing_thumbnails_by_ids(&priority_ids).await
                    {
                        if !priority_imgs.is_empty() {
                            let asset_ids: Vec<i64> =
                                priority_imgs.iter().map(|(id, _)| *id).collect();
                            if let Err(e) = db.increment_thumbnail_attempts_batch(&asset_ids).await
                            {
                                error!("Failed to pre-increment thumbnail attempts: {}", e);
                            }

                            // Insert LIFO into isolated queues (push FRONT)
                            for (id, path) in priority_imgs.into_iter().rev() {
                                let is_heavy =
                                    crate::thumbnails::get_thumbnail_strategy(Path::new(&path))
                                        .is_heavy();
                                if is_heavy {
                                    heavy_q.lock().await.push_front((id, path));
                                    notify_heavy.notify_one();
                                } else {
                                    light_q.lock().await.push_front((id, path));
                                    notify_light.notify_one();
                                }
                            }
                            did_work = true;
                        }
                    }
                }

                let light_len = light_q.lock().await.len();
                let heavy_len = heavy_q.lock().await.len();

                // 2. Fetch Regular queue only if we have room
                if light_len < 100 && heavy_len < 20 {
                    match db
                        .get_assets_needing_thumbnails(config.indexer_batch_size)
                        .await
                    {
                        Ok(imgs) => {
                            if !imgs.is_empty() {
                                let asset_ids: Vec<i64> = imgs.iter().map(|(id, _)| *id).collect();
                                if let Err(e) =
                                    db.increment_thumbnail_attempts_batch(&asset_ids).await
                                {
                                    error!("Failed to pre-increment thumbnail attempts: {}", e);
                                }

                                debug!("Dispatching {} assets needing thumbnails...", imgs.len());

                                // Insert FIFO into isolated queues (push BACK) - Background Tasks
                                for (id, path) in imgs {
                                    let is_heavy =
                                        crate::thumbnails::get_thumbnail_strategy(Path::new(&path))
                                            .is_heavy();
                                    if is_heavy {
                                        heavy_q.lock().await.push_back((id, path));
                                        notify_heavy.notify_one();
                                    } else {
                                        light_q.lock().await.push_back((id, path));
                                        notify_light.notify_one();
                                    }
                                }
                                did_work = true;
                            }
                        }
                        Err(database_error) => {
                            error!("Thumbnail worker DB error: {}", database_error);
                        }
                    }
                }

                if !did_work {
                    tokio::select! {
                        _ = token.cancelled() => break,
                        _ = sleep(Duration::from_millis(150)) => continue,
                    }
                } else {
                    tokio::task::yield_now().await;
                }
            }
        })
    }

    /// Isolated background worker for processing thumbnails continuously
    /// from a receiver, batching them into small chunks, and running a rayon pool.
    async fn worker_loop(
        queue: Arc<tokio::sync::Mutex<std::collections::VecDeque<(i64, String)>>>,
        notify: Arc<tokio::sync::Notify>,
        db: Arc<Db>,
        app: AppHandle,
        thumb_dir: PathBuf,
        token: CancellationToken,
        num_threads: usize,
        batch_size_limit: usize,
    ) {
        let pool = match rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
        {
            Ok(p) => Arc::new(p),
            Err(e) => {
                error!("Failed to create isolated thread pool: {}", e);
                return;
            }
        };

        loop {
            if token.is_cancelled() {
                break;
            }

            let mut batch = Vec::new();
            {
                let mut q = queue.lock().await;
                while batch.len() < batch_size_limit {
                    if let Some(item) = q.pop_front() {
                        batch.push(item);
                    } else {
                        break;
                    }
                }
            }

            if batch.is_empty() {
                tokio::select! {
                    _ = notify.notified() => continue,
                    _ = token.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_millis(500)) => continue,
                }
            }

            let pool_clone = pool.clone();
            let app_for_blocking = app.clone();
            let thumb_dir_clone = thumb_dir.clone();
            let thumb_dir_for_colors = thumb_dir.clone();

            let db_updates = tauri::async_runtime::spawn_blocking(move || {
                use rayon::prelude::*;
                pool_clone.install(|| {
                    batch
                        .par_iter()
                        .map(|(id, img_path)| {
                            let input_path = std::path::Path::new(&img_path);
                            if !input_path.exists() {
                                return (*id, Err("File not found".to_string()));
                            }

                            let thumb_name = crate::thumbnails::get_thumbnail_filename(img_path);
                            match crate::thumbnails::generate_thumbnail(
                                Some(&app_for_blocking),
                                input_path,
                                &thumb_dir_clone,
                                &thumb_name,
                                300,
                            ) {
                                Ok(fname) => (*id, Ok(fname)),
                                Err(e) => (*id, Err(e.to_string())),
                            }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .await
            .unwrap_or_default();

            // --- 🎨 COLOR EXTRACTION (post-thumbnail, non-blocking) ---
            let color_results: Vec<(i64, Option<Vec<AssetColor>>)> = db_updates
                .iter()
                .filter_map(|(id, result)| {
                    let Ok(filename) = result else { return None };

                    // Determine if this is an image asset by checking the original path
                    // We need a heuristic here since we only have the thumbnail filename
                    let thumbnail_full_path = thumb_dir_for_colors.join(filename);
                    if !thumbnail_full_path.exists() {
                        return None;
                    }

                    match crate::thumbnails::color_analysis::extract_color_palette(
                        &thumbnail_full_path,
                        None,
                    ) {
                        Ok(extracted_colors) => {
                            let asset_colors: Vec<AssetColor> = extracted_colors
                                .iter()
                                .enumerate()
                                .map(|(index, color)| AssetColor {
                                    id: 0,
                                    asset_id: *id,
                                    hex_color: color.hex_value.clone(),
                                    lab_lightness: color.lab_lightness,
                                    lab_green_red: color.lab_green_red,
                                    lab_blue_yellow: color.lab_blue_yellow,
                                    percentage: color.percentage,
                                    rank: (index + 1) as i32,
                                })
                                .collect();
                            Some((*id, Some(asset_colors)))
                        }
                        Err(_) => None, // Silently skip non-image or failed extractions
                    }
                })
                .collect();

            #[derive(serde::Serialize, Clone)]
            struct ThumbnailPayload {
                id: i64,
                path: String,
            }

            for (id, result) in db_updates {
                match result {
                    Ok(filename) => {
                        if let Err(database_error) = db.update_thumbnail_path(id, &filename).await {
                            tracing::error!("Error updating DB for thumbnail: {}", database_error);
                        } else {
                            let _ = app
                                .emit("thumbnail:ready", ThumbnailPayload { id, path: filename });

                            // Persist extracted colors for this asset (if available)
                            if let Some((_, Some(ref asset_colors))) =
                                color_results.iter().find(|(color_id, _)| *color_id == id)
                            {
                                if let Err(color_db_error) =
                                    db.insert_asset_colors(id, asset_colors).await
                                {
                                    warn!(
                                        "COLOR: Failed to save colors for asset {}: {}",
                                        id, color_db_error
                                    );
                                } else if let Some(dominant) = asset_colors.first() {
                                    let _ = db.update_dominant_color(id, &dominant.hex_color).await;
                                }
                            }
                        }
                    }
                    Err(err_msg) => {
                        tracing::error!("Thumbnail error for ID {}: {}", id, err_msg);
                        let _ = db.record_thumbnail_error(id, err_msg).await;
                    }
                }
            }
        }
    }
}
