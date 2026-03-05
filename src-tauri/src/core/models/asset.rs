use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The various lifecycle states of an asset in the Mundam system.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]

/// Enum representing the various lifecycle states of an asset in the Mundam system.
pub enum AssetState {
    /// FS Emit (Create/Rename)
    Discovered,
    /// Format-Kit is reading "Magic Bytes"
    Probing,
    /// Capability [Metadata] extraction succeeded
    Indexed,
    /// ThumbnailWorker extraction succeeded
    Thumbnailed,
    /// Status Gold/Perfect
    Idle,
    /// FS Emit "Modified" (External edit)
    Stale,
    /// HD Disconnected / Folder deleted from project
    Offline,
    /// Fallback for unidentified formats
    Unknown,
}

/// A Domain entity representing a media asset.
/// This is decoupled from the database storage format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub state: AssetState,
    pub format_type: String,
    pub family: String,
    pub file_size: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
