pub mod metadata;
pub mod types;
pub use types::*;
pub mod scan;
pub mod watcher;

use crate::db::Db;
use crate::lifecycle::LifecycleRegistry;
use std::sync::Arc;
use tauri::AppHandle;

/// Handles filesystem scanning and watcher lifecycle for indexed folders.
pub struct Indexer {
    /// Handle to the Tauri application.
    app_handle: AppHandle,
    /// Shared database connection.
    db: Arc<Db>,
    /// Registry of active filesystem watchers.
    registry: Arc<tokio::sync::Mutex<WatcherRegistry>>,
    /// Application lifecycle registry for tracking spawned tasks.
    lifecycle: Arc<LifecycleRegistry>,
}

impl Indexer {
    /// Create a new Indexer with the given application handle, database, watcher registry,
    /// and lifecycle registry.
    pub fn new(
        app_handle: AppHandle,
        db: &Db,
        registry: Arc<tokio::sync::Mutex<WatcherRegistry>>,
        lifecycle: Arc<LifecycleRegistry>,
    ) -> Self {
        Self {
            app_handle,
            db: Arc::new(Db {
                pool: db.pool.clone(),
            }),
            registry,
            lifecycle,
        }
    }

    /// Stop the filesystem watcher for the given root path by cancelling its token.
    pub async fn stop_watcher(&self, root_path: &str) {
        let path = normalize_path(root_path);
        let mut registry = self.registry.lock().await;
        if let Some(token) = registry.watchers.remove(&path) {
            println!("DEBUG: Stopping watcher for root: {}", path);
            token.cancel();
        }
    }

    /// Start a full scan of the given root path, followed by a filesystem watcher.
    pub async fn start_scan(&self, root_path: std::path::PathBuf) {
        scan::run_scan(
            self.app_handle.clone(),
            self.db.clone(),
            self.registry.clone(),
            self.lifecycle.clone(),
            root_path,
        )
        .await;
    }
}

fn normalize_path(path: &str) -> String {
    let p = path.trim_end_matches('/');
    if p.is_empty() {
        return "/".to_string();
    }
    p.to_string()
}
