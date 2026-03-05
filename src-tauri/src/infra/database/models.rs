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
    pub id: String,
    pub operation_type: String,
    pub asset_id: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error_note: Option<String>,
    pub created_at: DateTime<Utc>,
}
