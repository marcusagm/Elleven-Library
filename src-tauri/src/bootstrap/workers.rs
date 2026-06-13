//! Continuous Asynchronous Processing Orchestrator
//!
//! Allocates the fleets of reactive workers that handle heavy CPU-bound processing loads,
//! isolating these computations from Tauri IPC routes to prevent frontend bottlenecks.

use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Instantiates and activates queue consumers focused on feature extraction during idle time.
///
/// Initializes dedicated workers for iterative Thumbnail extraction (native resolution or via
/// bundled binaries) and the sub-worker for chromatic extraction.
///
/// # Arguments
/// * `app` - Reference to the Tauri AppHandle.
pub fn init(app: &AppHandle) {
    let dirs = app.state::<crate::bootstrap::AppDirectories>();
    let lifecycle = app.state::<Arc<crate::lifecycle::LifecycleRegistry>>().inner().clone();
    let format_registry = app.state::<Arc<crate::core::formats::FormatRegistry>>().inner().clone();
    let event_bus = app.state::<Arc<dyn crate::core::events::AppEventBus>>().inner().clone();
    let asset_ledger = app.state::<Arc<dyn crate::core::ledger::port::TransactionalAssetLedger>>().inner().clone();
    let asset_query_handler = app.state::<Arc<dyn crate::core::repository::AssetQueryHandler>>().inner().clone();

    let priority_state = std::sync::Arc::new(
        crate::core::workflows::thumbnails::priority::ThumbnailPriorityState::default(),
    );
    app.manage(priority_state.clone());

    // Start Thumbnail Worker (Hybrid Queue)
    let thumbnail_token = lifecycle.child_token();
    let thumbnail_worker = crate::processing::workers::thumbnail_worker::ThumbnailWorker::new(
        format_registry.clone(),
        asset_ledger.clone(),
        asset_query_handler.clone(),
        priority_state,
        dirs.thumbnails_dir.clone(),
    );
    let thumbnail_handle = thumbnail_worker.start(thumbnail_token.clone());
    lifecycle.register(
        "thumbnail_worker".to_string(),
        thumbnail_token,
        thumbnail_handle,
    );

    // Start Color Worker (Reactive to Thumbnails)
    let color_worker = crate::processing::workers::color_worker::ColorWorker::new(
        asset_ledger.clone(),
        event_bus.clone(),
        format_registry.clone(),
        dirs.thumbnails_dir.to_path_buf(),
    );
    color_worker.start();
}
