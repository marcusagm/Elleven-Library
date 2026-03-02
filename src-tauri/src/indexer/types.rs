use crate::db::models::AssetMetadata;
use serde::Serialize;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Serialize)]
pub struct ProgressPayload {
    pub total: usize,
    pub processed: usize,
    pub current_file: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct BatchChangePayload {
    pub added: Vec<AddedItemContext>,
    pub removed: Vec<RemovedItemContext>,
    pub updated: Vec<AddedItemContext>,
    pub needs_refresh: bool,
}

#[derive(Clone, Serialize, Debug)]
pub struct AddedItemContext {
    #[serde(flatten)]
    pub metadata: AssetMetadata,
    pub folder_id: i64,
    pub old_folder_id: Option<i64>,
}

#[derive(Clone, Serialize, Debug)]
pub struct RemovedItemContext {
    pub id: i64,
    pub folder_id: i64,
    pub tag_ids: Vec<i64>,
}

/// Struct to hold asset path with its parent directory path
pub struct IndexedAsset {
    pub metadata: AssetMetadata,
    pub parent_dir: String,
}

/// Registry of active filesystem watchers, keyed by root path.
///
/// Each watcher is associated with a `CancellationToken` that can be cancelled
/// to stop the watcher task cooperatively.
#[derive(Default)]
pub struct WatcherRegistry {
    /// Map from normalized root path to the cancellation token for that watcher.
    pub watchers: HashMap<String, CancellationToken>,
}
