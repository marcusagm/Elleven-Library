use crate::core::models::asset::AssetState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Payload for creating a new asset in the system.
///
/// This structure captures all mandatory information required by the Ledger
/// to formally register an asset in the database and filesystem registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetPayload {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// Initially detected file size in bytes.
    pub file_size: u64,
    /// Initially detected format (MimeType or Extension).
    pub format_type: String,
    /// The high-level family (IMAGE, VIDEO, etc).
    pub family: String,
    /// Initial state for the lifecycle machine (usually Discovered or Indexed).
    pub state_init: AssetState,
    /// Optional parent folder ID.
    pub folder_id: Option<String>,
}

/// Payload for updating tags associated with an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagsPayload {
    /// The unique identifier of the target asset.
    pub asset_id: String,
    /// List of tag names to be added.
    pub tags_to_add: Vec<String>,
    /// List of tag names to be removed.
    pub tags_to_remove: Vec<String>,
}

/// Payload for updating asset core identity (e.g., after a rename).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssetPayload {
    pub asset_id: Option<String>,
    pub old_path: Option<PathBuf>,
    pub new_path: PathBuf,
}

/// Payload for creating a new folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderPayload {
    pub parent_id: Option<String>,
    pub name: String,
    pub path: PathBuf,
}

/// Centralized Enum representing all mutation intentions (Commands) for the Asset Ledger.
///
/// Under CQRS, this represents the "Write" intent. The Ledger is responsible for
/// validating these commands against the current state before applying them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerCommand {
    /// Register a new asset in the system.
    CreateAsset(CreateAssetPayload),
    /// Register multiple assets in a single atomic transaction.
    BatchCreate(Vec<CreateAssetPayload>),
    /// Atomic update of asset tags.
    UpdateTags(UpdateTagsPayload),
    /// Update asset metadata (e.g., after a move/rename).
    UpdateAsset(UpdateAssetPayload),
    /// Mark an asset as stale (needs re-probing).
    MarkAsStale { asset_id: String },
    /// Formally delete an asset from the system.
    DeleteAsset {
        asset_id: Option<String>,
        path: Option<PathBuf>,
        /// If true, also attempts to delete the physical file.
        physical_delete: bool,
    },
    /// Create a new logical folder.
    CreateFolder(CreateFolderPayload),
    /// Assign an asset to a folder.
    SetAssetFolder {
        asset_id: String,
        folder_id: Option<String>,
    },
    /// Update an asset's thumbnail path and state.
    UpdateThumbnail {
        asset_id: String,
        thumbnail_path: String,
    },
}
