use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a saved search query acting as a virtual folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFolder {
    /// Unique identifier (UUID string)
    pub id: String,
    /// Display name of the smart folder
    pub name: String,
    /// JSON string representing the exact search criteria
    pub query_json: String,
    /// ISO-8601 creation timestamp
    pub created_at: DateTime<Utc>,
    /// ISO-8601 last update timestamp
    pub updated_at: DateTime<Utc>,
}
