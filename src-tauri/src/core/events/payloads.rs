use serde::Serialize;

/// Pure Domain Payload (Events).
///
/// Represents state changes and facts that occurred in the system.
/// Derives `Serialize` to allow sending events to the frontend via Tauri,
/// mapped as a TypeScript-friendly discriminated union.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
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
    AssetMetadataUpdated { asset_id: String },
    /// Request to re-extract colors for a specific asset
    ReextractAssetColors { asset_id: String },
    /// A thumbnail has been invalidated and needs regeneration.
    ThumbnailInvalidated { asset_id: String },
    /// The internal state (e.g., Processing -> Ready) of an asset has changed.
    AssetStateChanged {
        asset_id: String,
        old_state: String,
        new_state: String,
    },
    /// A new logical folder was created.
    FolderCreated {
        folder_id: String,
        parent_id: Option<String>,
        name: String,
        path: String,
    },
    /// A logical folder was removed.
    FolderRemoved {
        folder_id: String,
        path: String,
    },
    /// An asset was moved to a different logical folder.
    AssetFolderChanged {
        asset_id: String,
        folder_id: Option<String>,
    },

    // ├─ Tag CRUD Originated
    /// A new taxonomy tag was created.
    TagCreated { id: String, name: String },
    /// An existing tag's properties were updated.
    TagUpdated { id: String },
    /// A tag was deleted and removed from all assets.
    TagDeleted { id: String },

    // ├─ Smart Folders Originated
    /// A smart folder was created.
    SmartFolderCreated { id: String, name: String },
    /// A smart folder was updated.
    SmartFolderUpdated { id: String },
    /// A smart folder was deleted.
    SmartFolderDeleted { id: String },

    // ├─ OS Watcher Originated
    /// The Watcher detected a new file in the filesystem.
    FsFileDiscovered { path: String, size_bytes: u64 },
    /// The Watcher detected the removal of a path in the filesystem.
    FsPathDeleted { path: String },
    /// The Watcher detected a rename/move operation in the filesystem.
    FsPathRenamed { from: String, to: String },

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
    /// Progress update for an ongoing scan.
    ScanProgress {
        total: usize,
        processed: usize,
        current_file: String,
    },
    /// A library scan has completed.
    ScanCompleted { library_id: String },
    /// A new thumbnail has been generated and is ready at the given path.
    ThumbnailGenerated { asset_id: String, path: String },
}
