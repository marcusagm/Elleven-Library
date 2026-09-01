use crate::core::events::payloads::DomainEvent;
use crate::core::events::bus::AppEventBus;
use crate::core::repository::DuplicatesRepository;
use crate::core::models::DuplicateFingerprint;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Background worker that listens to domain events and generates
/// duplicates fingerprints for new assets.
pub struct DuplicateWorker {
    duplicates_repo: Arc<dyn DuplicatesRepository>,
    event_bus: Arc<dyn AppEventBus>,
}

impl DuplicateWorker {
    /// Creates a new instance of DuplicateWorker.
    pub fn new(
        duplicates_repo: Arc<dyn DuplicatesRepository>,
        event_bus: Arc<dyn AppEventBus>,
    ) -> Self {
        Self {
            duplicates_repo,
            event_bus,
        }
    }

    /// Starts the background listener loop.
    pub fn start(
        self: Arc<Self>,
        token: CancellationToken,
    ) -> tauri::async_runtime::JoinHandle<()> {
        let mut subscriber = self.event_bus.subscribe();

        tauri::async_runtime::spawn(async move {
            info!("DuplicateWorker: started and listening for DomainEvents");

            loop {
                tokio::select! {
                    recv_result = subscriber.recv() => {
                        match recv_result {
                            Ok(event) => {
                                self.handle_single_event(event).await;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped_count)) => {
                                warn!("DuplicateWorker: lagged behind {} events", skipped_count);
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                error!("DuplicateWorker subscriber channel closed. Exiting.");
                                break;
                            }
                        }
                    }
                    _ = token.cancelled() => {
                        info!("DuplicateWorker: Cancelled via token. Shutting down.");
                        break;
                    }
                }
            }
        })
    }

    /// Process a single event. 
    /// Dispatches blocking operations to `spawn_blocking` to avoid blocking the async executor.
    async fn handle_single_event(&self, event: DomainEvent) {
        match event {
            DomainEvent::AssetCreated { asset_id, path, format } => {
                let asset_id_clone = asset_id.clone();
                let path_clone = path.clone();
                
                // Spawn a blocking task to perform heavy CPU/IO hashing
                let fingerprint_res = tokio::task::spawn_blocking(move || {
                    Self::generate_fingerprint(&asset_id_clone, &path_clone, &format)
                }).await;

                match fingerprint_res {
                    Ok(Ok(fingerprint)) => {
                        if let Err(e) = self.duplicates_repo.save_fingerprint(fingerprint).await {
                            error!("DuplicateWorker: failed to save fingerprint for {}: {}", asset_id, e);
                        } else {
                            info!("DuplicateWorker: successfully generated and saved fingerprint for {}", asset_id);
                            
                            // Immediately run the exact match scan to auto-group if enabled
                            if let Err(e) = self.duplicates_repo.run_exact_match_scan().await {
                                warn!("DuplicateWorker: failed to auto-group exact matches: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("DuplicateWorker: error generating fingerprint for {}: {}", asset_id, e);
                    }
                    Err(e) => {
                        error!("DuplicateWorker: blocking task panicked or was cancelled for {}: {}", asset_id, e);
                    }
                }
            }
            // For future: Handle AssetDeleted to clean up fingerprints, etc.
            _ => {}
        }
    }

    /// Synchronous function to read the file and compute hashes.
    /// Runs inside `spawn_blocking`.
    fn generate_fingerprint(
        asset_id: &str,
        path: &str,
        format: &str,
    ) -> Result<DuplicateFingerprint, String> {
        // Read file size
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
            
        let file_size = metadata.len() as i64;

        // TODO: Implement actual hashing logic (Blake3, Phash, etc.)
        // This is a placeholder for the actual extraction pipeline.
        let content_hash = Some(format!("hash_{}", file_size)); 
        
        Ok(DuplicateFingerprint {
            asset_id: asset_id.to_string(),
            content_hash,
            perceptual_hash: None,
            block_hash: None,
            thumb_hash: None,
            width: None,
            height: None,
            file_size: Some(file_size),
            mime_type: None,
            format_family: Some(format.to_string()),
            color_profile: None,
            orientation: None,
            fingerprint_version: 1,
            updated_at: chrono::Utc::now(),
        })
    }
}
