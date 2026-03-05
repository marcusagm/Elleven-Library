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

/// Centralized Enum representing all mutation intentions (Commands) for the Asset Ledger.
///
/// Under CQRS, this represents the "Write" intent. The Ledger is responsible for
/// validating these commands against the current state before applying them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerCommand {
    /// Register a new asset in the system.
    CreateAsset(CreateAssetPayload),
    /// Atomic update of asset tags.
    UpdateTags(UpdateTagsPayload),
    /// Mark an asset as stale (needs re-probing).
    MarkAsStale { asset_id: String },
    /// Formally delete an asset from the system.
    DeleteAsset {
        asset_id: String,
        /// If true, also attempts to delete the physical file.
        physical_delete: bool,
    },
}
