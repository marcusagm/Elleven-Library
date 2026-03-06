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
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Name of the asset
    pub name: String,
    /// Path to the asset
    pub path: PathBuf,
    /// State of the asset
    pub state: AssetState,
    /// Format type of the asset
    pub format_type: String,
    /// Family of the asset
    pub family: String,
    /// File size of the asset
    pub file_size: u64,
    /// Timestamp of when the asset was created
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of when the asset was updated
    pub updated_at: Option<DateTime<Utc>>,

    /// Width of the asset
    pub width: Option<i32>,
    /// Height of the asset
    pub height: Option<i32>,
    /// Duration of the asset in seconds
    pub duration_secs: Option<f64>,
    /// Technical payload of the asset
    pub technical_payload: Option<serde_json::Value>,
    /// Semantic payload of the asset
    pub semantic_payload: Option<serde_json::Value>,
    /// Dominant colors of the asset
    pub dominant_colors: Option<serde_json::Value>,
    /// Reference to the parent folder in recursive tree
    pub folder_id: Option<String>,
}

/// A Domain entity representing a recursive folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// ID of the parent folder
    pub parent_id: Option<String>,
    /// Name of the folder
    pub name: String,
    /// Path of the folder
    pub path: PathBuf,
    /// Timestamp of when the folder was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the folder was last updated
    pub updated_at: DateTime<Utc>,
}

/// A Domain entity representing a taxonomy Tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Name of the tag
    pub name: String,
    /// Color of the tag
    pub color: Option<String>,
    /// ID of the parent tag
    pub parent_id: Option<String>,
}

/// A lightweight projection of an asset for grid listings and infinite scroll.
/// Focuses on visual performance and minimal bridge overhead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummaryDto {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Name of the asset
    pub name: String,
    /// State of the asset
    pub state: AssetState,
    /// Format type of the asset
    pub format_type: String,
    /// Family of the asset
    pub family: String,
    /// Timestamp of when the asset was created
    pub created_at: Option<DateTime<Utc>>,
    /// Parent folder ID
    pub folder_id: Option<String>,
}

/// Parameters for filtering asset listings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetFilter {
    /// Family of the asset
    pub family: Option<String>,
    /// State of the asset
    pub state: Option<AssetState>,
    /// Search query
    pub search_query: Option<String>,
    /// Filter by folder
    pub folder_id: Option<String>,
    /// Filter by tags (any of)
    pub tags: Option<Vec<String>>,
}

/// Pagination parameters for the read model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageParams {
    /// Page number
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Default implementation for PageParams.
impl Default for PageParams {
    /// Returns the default PageParams.
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 50,
        }
    }
}

/// Implementation of the PageParams struct.
impl PageParams {
    /// Returns the OFFSET for SQL queries.
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.page_size
    }

    /// Returns the LIMIT for SQL queries.
    pub fn limit(&self) -> u32 {
        self.page_size
    }
}
