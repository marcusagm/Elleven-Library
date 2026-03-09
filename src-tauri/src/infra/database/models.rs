use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database model for the main assets table in V2.
///
/// This struct directly maps to the `assets` table in the SQLite database.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]

/// Struct that represents an asset in the database.
pub struct AssetDb {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Display name of the asset
    pub name: String,
    /// Absolute filesystem path
    pub path: String,
    /// Current state in the lifecycle machine
    pub state: String,
    /// Detected format (e.g. image/png)
    pub format_type: String,
    /// Media family (e.g. IMAGE, VIDEO)
    pub family: String,
    /// Size in bytes
    pub file_size: i64,
    /// File creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// File modification timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Parent folder ID
    pub folder_id: Option<String>,
    /// Path to the generated thumbnail file
    pub thumbnail_path: Option<String>,

    /// Width of the asset
    pub width: Option<i32>,
    /// Height of the asset
    pub height: Option<i32>,
    /// Duration of the asset in seconds
    pub duration_secs: Option<f64>,
    /// Dominant colors of the asset
    pub dominant_color: Option<serde_json::Value>,
    /// Technical payload of the asset
    pub technical_payload: Option<serde_json::Value>,
    /// Semantic payload of the asset
    pub semantic_payload: Option<serde_json::Value>,
}

/// Lightweight database projection for asset listings.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssetSummaryDb {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Display name of the asset
    pub name: String,
    /// Current state in the lifecycle machine
    pub state: String,
    /// Detected format (e.g. image/png)
    pub format_type: String,
    /// Media family (e.g. IMAGE, VIDEO)
    pub family: String,
    /// Timestamp of when the asset was created
    pub created_at: Option<DateTime<Utc>>,
    /// ID of the parent folder
    pub folder_id: Option<String>,
}

/// Dynamic metadata envelope for specific format capabilities.
///
/// Complements the core asset data with format-specific properties.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]

/// Struct that represents the metadata of an asset in the database.
pub struct AssetMetadataEnvelopeDb {
    /// Unique identifier (UUID/ULID)
    pub asset_id: String,
    /// Width of the asset
    pub width: Option<i32>,
    /// Height of the asset
    pub height: Option<i32>,
    /// Duration of the asset in seconds
    pub duration_secs: Option<f64>,
    /// Dominant colors of the asset
    pub dominant_color: Option<serde_json::Value>,
    /// Technical payload of the asset
    pub technical_payload: Option<serde_json::Value>,
    /// Semantic payload of the asset
    pub semantic_payload: Option<serde_json::Value>,
}

/// Record of an operation performed on an asset, used for audit and undo.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]

/// Struct that represents a record of an operation performed on an asset in the database.
pub struct AssetOperationLogDb {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// Type of operation (e.g. "tag", "move", "delete")
    pub operation_type: String,
    /// ID of the asset involved
    pub asset_id: String,
    /// Payload containing operation details
    pub payload: serde_json::Value,
    /// Status of the operation (e.g. "pending", "completed", "failed")
    pub status: String,
    /// Error note if the operation failed
    pub error_note: Option<String>,
    /// Timestamp of when the operation was created
    pub created_at: DateTime<Utc>,
}

/// Database model for asset colors.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssetColorDb {
    pub id: i64,
    pub asset_id: String,
    pub hex_color: String,
    pub lab_lightness: f64,
    pub lab_green_red: f64,
    pub lab_blue_yellow: f64,
    pub percentage: f64,
    pub rank: i32,
}

impl From<crate::core::models::asset::AssetColor> for AssetColorDb {
    fn from(color: crate::core::models::asset::AssetColor) -> Self {
        Self {
            id: color.id.unwrap_or(0),
            asset_id: String::new(), // Will be populated during batch insert
            hex_color: color.hex_color,
            lab_lightness: color.lab_lightness,
            lab_green_red: color.lab_green_red,
            lab_blue_yellow: color.lab_blue_yellow,
            percentage: color.percentage,
            rank: color.rank,
        }
    }
}

/// Converts an AssetDb to an Asset.
///
/// # Arguments
///
/// * `row` - The AssetDb to convert.
///
/// # Returns
///
/// An Asset.
impl From<AssetDb> for crate::core::models::Asset {
    /// Converts an AssetDb to an Asset.
    ///
    /// # Arguments
    ///
    /// * `row` - The AssetDb to convert.
    ///
    /// # Returns
    ///
    /// An Asset.
    fn from(row: AssetDb) -> Self {
        use crate::core::models::asset::AssetState;
        use std::str::FromStr;
        Self {
            id: row.id,
            name: row.name,
            path: std::path::PathBuf::from(row.path),
            state: AssetState::from_str(&row.state).unwrap_or(AssetState::Unknown),
            format_type: row.format_type,
            family: row.family,
            file_size: row.file_size as u64,
            created_at: row.created_at,
            updated_at: row.updated_at,
            folder_id: row.folder_id,
            width: row.width,
            height: row.height,
            duration_secs: row.duration_secs,
            technical_payload: row.technical_payload,
            semantic_payload: row.semantic_payload,
            dominant_color: row.dominant_color,
            thumbnail_path: row.thumbnail_path,
        }
    }
}

/// Converts an AssetSummaryDb to an AssetSummaryDto.
///
/// # Arguments
///
/// * `row` - The AssetSummaryDb to convert.
///
/// # Returns
///
/// An AssetSummaryDto.
impl From<AssetSummaryDb> for crate::core::models::AssetSummaryDto {
    /// Converts an AssetSummaryDb to an AssetSummaryDto.
    ///
    /// # Arguments
    ///
    /// * `row` - The AssetSummaryDb to convert.
    ///
    /// # Returns
    ///
    /// An AssetSummaryDto.
    fn from(row: AssetSummaryDb) -> Self {
        use crate::core::models::asset::AssetState;
        use std::str::FromStr;
        Self {
            id: row.id,
            name: row.name,
            state: AssetState::from_str(&row.state).unwrap_or(AssetState::Unknown),
            format_type: row.format_type,
            family: row.family,
            created_at: row.created_at,
            folder_id: row.folder_id,
        }
    }
}

/// Database model for folders.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FolderDb {
    /// Unique identifier (UUID/ULID)
    pub id: String,
    /// ID of the parent folder
    pub parent_id: Option<String>,
    /// Name of the folder
    pub name: String,
    /// Path of the folder
    pub path: String,
    /// Timestamp of when the folder was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the folder was last updated
    pub updated_at: DateTime<Utc>,
}

impl From<FolderDb> for crate::core::models::asset::Folder {
    /// Converts a FolderDb to a Folder.
    ///
    /// # Arguments
    ///
    /// * `row` - The FolderDb to convert.
    ///
    /// # Returns
    ///
    /// A Folder.
    fn from(row: FolderDb) -> Self {
        Self {
            id: row.id,
            parent_id: row.parent_id,
            name: row.name,
            path: std::path::PathBuf::from(row.path),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Database model for tags.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TagDb {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>,
}

/// Converts a TagDb to a Tag.
///
/// # Arguments
///
/// * `row` - The TagDb to convert.
///
/// # Returns
///
/// A Tag.
impl From<TagDb> for crate::core::models::asset::Tag {
    fn from(row: TagDb) -> Self {
        Self {
            id: row.id,
            name: row.name,
            color: row.color,
            parent_id: row.parent_id,
        }
    }
}
