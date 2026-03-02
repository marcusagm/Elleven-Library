use crate::db::Db;
use crate::error::AppResult;
use crate::indexer::Indexer;
use crate::lifecycle::LifecycleRegistry;
use std::path::PathBuf;
use tauri::Manager;

/// Start indexing a directory.
///
/// # Errors
/// Returns error if database or watcher registry is not initialized.
#[tauri::command]
pub async fn start_indexing(path: String, app: tauri::AppHandle) -> AppResult<()> {
    tracing::info!("COMMAND: start_indexing called with path: {}", path);

    // Get DB from state with safety
    let db = app
        .try_state::<std::sync::Arc<Db>>()
        .ok_or_else(|| crate::error::AppError::Internal("Database not initialized".to_string()))?;

    let registry = app
        .try_state::<std::sync::Arc<tokio::sync::Mutex<crate::indexer::WatcherRegistry>>>()
        .ok_or_else(|| crate::error::AppError::Internal("Registry not initialized".to_string()))?;

    let lifecycle = app
        .try_state::<std::sync::Arc<LifecycleRegistry>>()
        .ok_or_else(|| crate::error::AppError::Internal("Lifecycle not initialized".to_string()))?;

    let indexer = Indexer::new(
        app.clone(),
        db.inner(),
        registry.inner().clone(),
        lifecycle.inner().clone(),
    );

    let root = PathBuf::from(path);
    indexer.start_scan(root).await;
    Ok(())
}
