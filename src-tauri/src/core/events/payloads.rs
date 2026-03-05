use serde::Serialize;

/// Pure Domain Payload (Events).
///
/// Represents state changes and facts that occurred in the system.
/// Derives `Serialize` to allow sending events to the frontend via Tauri.
#[derive(Clone, Debug, Serialize)]
pub enum DomainEvent {
    // ├─ Ledger Originated
    /// A new asset was formally created in the database and on disk.
    AssetCreated {
        asset_id: String,
        path: String,
        format: String,
    },
    /// An asset's tags have been changed.
    AssetTagsUpdated {
        asset_id: String,
        active_tags: Vec<String>,
    },
    /// The internal state (e.g., Processing -> Ready) of an asset has changed.
    AssetStateChanged {
        asset_id: String,
        old_state: String,
        new_state: String,
    },

    // ├─ OS Watcher Originated
    /// The Watcher detected a new file in the filesystem.
    FsFileDiscovered { path: String, size_bytes: u64 },
    /// The Watcher detected the removal of a path in the filesystem.
    FsPathDeleted { path: String },

    // ├─ Workers/Jobs Originated (Heavy Extractor Lifecycle)
    /// An extraction (Thumbnail, Metadata, etc.) was completed successfully.
    ExtractionCompleted {
        asset_id: String,
        capability: String,
    },
    /// A background job failed.
    JobFailed {
        asset_id: String,
        error_reason: String,
    },

    // ├─ System
    /// A library scan has started.
    ScanStarted { library_id: String },
    /// A library scan has completed.
    ScanCompleted { library_id: String },
}
