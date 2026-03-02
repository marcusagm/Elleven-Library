//! Database models and shared structures for the Mundam application.
//!
//! this module defines the data transfer objects (DTOs) and database entities
//! used across the backend and returned to the frontend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single asset record in the database.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AssetMetadata {
    /// Unique identifier for the asset.
    pub id: i64,
    /// Absolute filesystem path to the original asset.
    pub path: String,
    /// Filename with extension.
    pub filename: String,
    /// Image width in pixels, if detectable.
    pub width: Option<i32>,
    /// Image height in pixels, if detectable.
    pub height: Option<i32>,
    /// File size in bytes.
    pub size: i64,
    /// Primary file extension or detected format.
    pub format: String,
    /// General media type category (e.g., Image, Video, Project).
    pub media_type: String,
    /// Optional path to the generated thumbnail.
    pub thumbnail_path: Option<String>,
    /// User-assigned rating (e.g., 0 to 5).
    #[sqlx(default)]
    pub rating: i32,
    /// Optional user notes or description.
    #[sqlx(default)]
    pub notes: Option<String>,
    /// Last modification time of the file.
    pub modified_at: DateTime<Utc>,
    /// Creation time of the file.
    pub created_at: DateTime<Utc>,
    /// Time when the asset was first indexed by Mundam.
    #[sqlx(default)]
    pub added_at: Option<DateTime<Utc>>,
}

/// A categorization tag that can be applied to assets.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    /// Unique identifier for the tag.
    pub id: i64,
    /// Display name of the tag.
    pub name: String,
    /// Optional parent tag ID for hierarchical organization.
    pub parent_id: Option<i64>,
    /// Optional hexadecimal color associated with the tag.
    pub color: Option<String>,
    /// Sorting order index.
    #[sqlx(default)]
    pub order_index: i64,
}

/// Count of assets associated with a specific tag.
#[derive(Debug, Serialize, Deserialize)]
pub struct TagCount {
    pub tag_id: i64,
    pub count: i64,
}

/// Count of assets within a specific folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderCount {
    pub folder_id: i64,
    pub count: i64,
}

/// Comprehensive statistics about the library.
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryStats {
    /// Total number of assets in the library.
    pub total_assets: i64,
    /// Number of assets that have no tags assigned.
    pub untagged_assets: i64,
    /// Distribution of assets across tags.
    pub tag_counts: Vec<TagCount>,
    /// Direct asset counts per folder.
    pub folder_counts: Vec<FolderCount>,
    /// Asset counts per folder including all subfolders.
    pub folder_counts_recursive: Vec<FolderCount>,
}

/// A saved search filter that acts like a dynamic folder.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SmartFolder {
    /// Unique identifier for the smart folder.
    pub id: i64,
    /// Display name of the smart folder.
    pub name: String,
    /// JSON string representing the search criteria.
    pub query_json: String,
    /// ISO-8601 creation timestamp.
    pub created_at: DateTime<Utc>,
}
