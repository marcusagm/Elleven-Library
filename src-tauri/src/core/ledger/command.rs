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

/// Payload for updating an asset's color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssetColorsPayload {
    /// The unique identifier of the target asset.
    pub asset_id: String,
    /// The complete list of extracted colors.
    pub colors: Vec<crate::core::models::asset::AssetColor>,
}

/// Payload for creating a new taxonomy tag.
///
/// The Ledger will generate a UUID for the tag and persist it in the tags table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagPayload {
    /// Display name of the tag (must be unique).
    pub name: String,
    /// Optional parent tag ID for hierarchical organization.
    pub parent_id: Option<String>,
    /// Optional hex color code for the tag.
    pub color: Option<String>,
}

/// Payload for updating an existing taxonomy tag's properties.
///
/// Only non-None fields will be applied in the UPDATE statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTagPayload {
    /// The unique identifier of the tag to update.
    pub id: String,
    /// New display name (optional).
    pub name: Option<String>,
    /// New hex color (optional).
    pub color: Option<String>,
    /// New parent tag ID (optional; use Some("") or None logic to unparent).
    pub parent_id: Option<String>,
    /// New sorting order index (optional).
    pub order_index: Option<i64>,
}

/// Payload for batch tag-asset association operations.
///
/// Used by AddTagsToAssetsBatch, RemoveTagsFromAssetsBatch,
/// and ReplaceTagsForAssetsBatch commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTagsPayload {
    /// The asset IDs to operate on.
    pub asset_ids: Vec<String>,
    /// The tag IDs to add/remove/replace.
    pub tag_ids: Vec<String>,
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
    /// Update an asset's color palette.
    UpdateAssetColors(UpdateAssetColorsPayload),

    /// Create a new taxonomy tag with name, optional color and parent.
    CreateTag(CreateTagPayload),
    /// Update an existing tag's properties (name, color, parent_id, order_index).
    UpdateTag(UpdateTagPayload),
    /// Delete a tag and remove all its asset associations.
    DeleteTag { id: String },
    /// Associate multiple tags with multiple assets in a single transaction.
    AddTagsToAssetsBatch(BatchTagsPayload),
    /// Remove specific tag associations from multiple assets.
    RemoveTagsFromAssetsBatch(BatchTagsPayload),
    /// Replace all tag associations for multiple assets with a new set.
    ReplaceTagsForAssetsBatch(BatchTagsPayload),
}
