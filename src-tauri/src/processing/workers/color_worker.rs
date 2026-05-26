//! Color Worker - Automated background extraction of color palettes.
//!
//! Listens for `DomainEvent::ThumbnailGenerated`, analyzes the thumbnail file,
//! and dispatches a `LedgerCommand::UpdateAssetColors` to persist the result.

use crate::core::events::{AppEventBus, DomainEvent};
use crate::core::formats::FormatRegistry;
use crate::core::ledger::command::{LedgerCommand, UpdateAssetColorsPayload};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::feature::analysis::colors::extract_color_palette;
use crate::processing::media::image_utils;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// Background worker that monitors thumbnail generation and extracts colors.
pub struct ColorWorker {
    ledger: Arc<dyn TransactionalAssetLedger>,
    event_bus: Arc<dyn AppEventBus>,
    format_registry: Arc<FormatRegistry>,
    thumbnails_dir: PathBuf,
}

/// Implementation of ColorWorker.
impl ColorWorker {
    /// Creates a new ColorWorker.
    ///
    /// # Arguments
    ///
    /// * `ledger` - The transactional asset ledger.
    /// * `event_bus` - The application event bus.
    /// * `thumbnails_dir` - The directory containing thumbnails.
    pub fn new(
        ledger: Arc<dyn TransactionalAssetLedger>,
        event_bus: Arc<dyn AppEventBus>,
        format_registry: Arc<FormatRegistry>,
        thumbnails_dir: PathBuf,
    ) -> Self {
        Self {
            ledger,
            event_bus,
            format_registry,
            thumbnails_dir,
        }
    }

    /// Starts the color worker loop.
    ///
    /// This method subscribes to the event bus and listens for `ThumbnailGenerated` events.
    /// When an event is received, it extracts the color palette from the thumbnail and
    /// persists it to the ledger.
    ///
    /// # Arguments
    ///
    /// * `self` - The ColorWorker instance.
    ///
    /// # Returns
    ///
    /// None
    pub fn start(self) {
        let mut subscriber = self.event_bus.subscribe();
        let ledger = self.ledger.clone();
        let thumb_dir = self.thumbnails_dir.clone();
        let _format_registry = self.format_registry.clone();

        tokio::spawn(async move {
            info!("WORKER: ColorWorker started and listening for ThumbnailGenerated");

            loop {
                match subscriber.recv().await {
                    Ok(event) => {
                        if let DomainEvent::ThumbnailGenerated {
                            asset_id,
                            path,
                            format: _,
                        } = event
                        {
                            let asset_id_clone = asset_id.clone();
                            if path.is_empty() {
                                debug!(
                                    "WORKER: Skipping color extraction for asset {} - No thumbnail generated",
                                    asset_id_clone
                                );
                                continue;
                            }

                            // STARK: Reliability Check - Thumbnails are always generated as valid WebP if successful.
                            // We extract from the thumbnail directly to ensure compatibility across all formats.
                            let thumb_path = thumb_dir.join(&path);
                            let ledger_clone = ledger.clone();

                            if !thumb_path.exists() || !thumb_path.is_file() {
                                error!(
                                    "WORKER: Color extraction failed - Thumbnail not found or invalid at {:?}",
                                    thumb_path
                                );
                                continue;
                            }

                            // VALIDATION: Check if the thumbnail has a valid header before processing
                            if let Ok(bytes) = std::fs::read(&thumb_path) {
                                if !image_utils::is_valid_image(&bytes) {
                                    error!(
                                        "WORKER: Color extraction aborted - Thumbnail {:?} has invalid magic bytes/header",
                                        thumb_path
                                    );
                                    continue;
                                }
                            }

                            info!(
                                "WORKER: Starting color extraction for asset {}",
                                asset_id_clone
                            );

                            // Perform heavy k-means in a blocking thread to avoid choking the async reactor
                            let result = tokio::task::spawn_blocking(move || {
                                extract_color_palette(&thumb_path, None)
                            })
                            .await;

                            match result {
                                Ok(Ok(colors)) => {
                                    info!(
                                        "WORKER: Color extraction successful for asset {} (found {} colors)",
                                        asset_id_clone,
                                        colors.len()
                                    );

                                    let command = LedgerCommand::UpdateAssetColors(
                                        UpdateAssetColorsPayload {
                                            asset_id: asset_id_clone.clone(),
                                            colors,
                                        },
                                    );

                                    if let Err(e) = ledger_clone.execute(command).await {
                                        error!(
                                            "WORKER: Failed to persist colors for asset {}: {}",
                                            asset_id_clone, e
                                        );
                                    } else {
                                        info!(
                                            "WORKER: Successfully persisted colors for asset {}",
                                            asset_id_clone
                                        );
                                    }
                                }
                                Ok(Err(e)) => {
                                    error!(
                                        "WORKER: Color analysis failed for asset {}: {}",
                                        asset_id_clone, e
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "WORKER: ColorWorker task panicked for asset {}: {}",
                                        asset_id_clone, e
                                    );
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        error!("WORKER: ColorWorker lagged behind {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("WORKER: ColorWorker subscriber channel closed. Exiting.");
                        break;
                    }
                }
            }
        });
    }
}
