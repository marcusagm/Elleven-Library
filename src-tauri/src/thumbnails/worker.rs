use crate::db::Db;
use crate::thumbnails::priority::ThumbnailPriorityState;
use crate::thumbnails::{generate_thumbnail, get_thumbnail_filename};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

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

        tauri::async_runtime::spawn(async move {
            loop {
                // Check for cancellation before starting a new batch
                if token.is_cancelled() {
                    println!("LIFECYCLE: ThumbnailWorker shutting down (token cancelled)");
                    break;
                }

                // 1. Check Priority Queue First
                let priority_ids = priority_state
                    .priority_ids
                    .lock()
                    .unwrap_or_else(|poisoned_error| poisoned_error.into_inner())
                    .iter()
                    .cloned()
                    .collect::<Vec<i64>>();

                let mut assets = Vec::new();
                let mut is_priority_batch = false;

                if !priority_ids.is_empty() {
                    if let Ok(priority_imgs) =
                        db.get_assets_needing_thumbnails_by_ids(&priority_ids).await
                    {
                        if !priority_imgs.is_empty() {
                            assets = priority_imgs;
                            is_priority_batch = true;
                        }
                    }
                }

                // 2. If no priority work, check regular queue
                if assets.is_empty() {
                    match db
                        .get_assets_needing_thumbnails(config.indexer_batch_size)
                        .await
                    {
                        Ok(imgs) => {
                            assets = imgs;
                        }
                        Err(database_error) => {
                            eprintln!("Thumbnail worker DB error: {}", database_error);
                            // Wait before retrying, but respect cancellation
                            tokio::select! {
                                _ = token.cancelled() => {
                                    println!("LIFECYCLE: ThumbnailWorker shutting down during error backoff");
                                    break;
                                }
                                _ = sleep(Duration::from_secs(10)) => { continue; }
                            }
                        }
                    }
                }

                if assets.is_empty() {
                    // No work — wait briefly, but respect cancellation
                    tokio::select! {
                        _ = token.cancelled() => {
                            println!("LIFECYCLE: ThumbnailWorker shutting down (idle)");
                            break;
                        }
                        _ = sleep(Duration::from_secs(2)) => { continue; }
                    }
                }

                if !is_priority_batch {
                    println!(
                        "DEBUG: Found {} assets needing thumbnails. Starting batch...",
                        assets.len()
                    );
                }

                // Pre-increment attempts for all assets in the batch to prevent poison pills
                // from catching the worker in an infinite crash loop spanning restarts.
                let asset_ids: Vec<i64> = assets.iter().map(|(id, _)| *id).collect();
                if let Err(increment_error) =
                    db.increment_thumbnail_attempts_batch(&asset_ids).await
                {
                    eprintln!(
                        "Failed to pre-increment thumbnail attempts: {}",
                        increment_error
                    );
                }

                // Clone thumb_dir for the move closure
                let thumb_dir_clone = thumb_dir.clone();
                let num_threads = config.thumbnail_threads;
                let app_for_blocking = app.clone();

                // Use a blocking thread for CPU-intensive work
                let db_updates = tauri::async_runtime::spawn_blocking(move || {
                    use rayon::prelude::*;
                    use rayon::ThreadPoolBuilder;

                    // Create a limited thread pool
                    let pool = match ThreadPoolBuilder::new().num_threads(num_threads).build() {
                        Ok(thread_pool) => thread_pool,
                        Err(pool_error) => {
                            eprintln!("Failed to create thread pool: {}", pool_error);
                            return Vec::new();
                        }
                    };

                    pool.install(|| {
                        assets
                            .par_iter()
                            .map(|(id, img_path)| {
                                let input_path = Path::new(&img_path);
                                if !input_path.exists() {
                                    return (*id, Err("File not found".to_string()));
                                }

                                let thumb_name = get_thumbnail_filename(img_path);

                                // Generate thumbnail
                                match generate_thumbnail(
                                    Some(&app_for_blocking),
                                    input_path,
                                    &thumb_dir_clone,
                                    &thumb_name,
                                    300,
                                ) {
                                    Ok(generated_filename) => (*id, Ok(generated_filename)),
                                    Err(generation_error) => {
                                        (*id, Err(generation_error.to_string()))
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .await
                .unwrap_or_else(|join_error| {
                    eprintln!("Blocking task failed: {}", join_error);
                    Vec::new()
                });

                #[derive(serde::Serialize, Clone)]
                struct ThumbnailPayload {
                    id: i64,
                    path: String,
                }

                // Perform DB updates sequentially (async)
                let mut error_count = 0;
                let total_items = db_updates.len();

                for (id, result) in db_updates {
                    match result {
                        Ok(filename) => {
                            if let Err(update_error) = db.update_thumbnail_path(id, &filename).await
                            {
                                eprintln!("Error updating DB for thumbnail: {}", update_error);
                                error_count += 1;
                            } else {
                                let payload = ThumbnailPayload {
                                    id,
                                    path: filename.clone(),
                                };
                                let _ = app.emit("thumbnail:ready", payload);
                            }
                        }
                        Err(err_msg) => {
                            eprintln!("Thumbnail error for ID {}: {}", id, err_msg);
                            error_count += 1;
                            if let Err(record_error) = db.record_thumbnail_error(id, err_msg).await
                            {
                                eprintln!(
                                    "Failed to record thumbnail error in DB: {}",
                                    record_error
                                );
                            }
                        }
                    }
                }

                // Brief yield between batches with dynamic backoff for massive failures
                if !is_priority_batch {
                    if total_items > 0 && error_count == total_items {
                        // 100% failure rate: back off significantly (e.g. out of memory, disk full, etc.)
                        println!(
                            "WARN: 100% failure rate in thumbnail batch. Backing off for 10s..."
                        );
                        sleep(Duration::from_secs(10)).await;
                    } else if total_items > 0 && error_count > total_items / 2 {
                        // > 50% failure rate: slow down to prevent thrashing
                        println!(
                            "WARN: >50% failure rate in thumbnail batch. Backing off for 5s..."
                        );
                        sleep(Duration::from_secs(5)).await;
                    } else {
                        // Normal brief yield
                        sleep(Duration::from_millis(100)).await;
                    }
                } else {
                    sleep(Duration::from_millis(10)).await;
                }
            }
        })
    }
}
