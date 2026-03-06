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

    // Joined metadata (Optional)
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_secs: Option<f64>,
    pub technical_payload: Option<serde_json::Value>,
    pub semantic_payload: Option<serde_json::Value>,
}

/// Lightweight database projection for asset listings.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssetSummaryDb {
    pub id: String,
    pub name: String,
    pub state: String,
    pub format_type: String,
    pub family: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Dynamic metadata envelope for specific format capabilities.
///
/// Complements the core asset data with format-specific properties.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]

/// Struct that represents the metadata of an asset in the database.
pub struct AssetMetadataEnvelopeDb {
    pub asset_id: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_secs: Option<f64>,
    pub dominant_colors: Option<serde_json::Value>,
    pub technical_payload: Option<serde_json::Value>,
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
            width: row.width,
            height: row.height,
            duration_secs: row.duration_secs,
            technical_payload: row.technical_payload,
            semantic_payload: row.semantic_payload,
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
        }
    }
}
