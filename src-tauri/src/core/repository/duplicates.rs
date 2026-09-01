use crate::core::error::AppResult;
use crate::core::models::{DuplicateFingerprint, DuplicateGroup, DuplicateCandidate, DuplicateRuleSet, DuplicateResolution};
use async_trait::async_trait;

/// Port for read and write operations related to duplicates.
#[async_trait]
pub trait DuplicatesRepository: Send + Sync {
    /// Saves or updates a duplicate fingerprint for an asset.
    ///
    /// # Arguments
    /// * `fingerprint` - The fingerprint data to save.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the insert fails.
    async fn save_fingerprint(&self, fingerprint: DuplicateFingerprint) -> AppResult<()>;

    /// Retrieves a duplicate fingerprint by asset ID.
    ///
    /// # Arguments
    /// * `asset_id` - The unique identifier of the asset.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the query fails.
    async fn get_fingerprint(&self, asset_id: &str) -> AppResult<Option<DuplicateFingerprint>>;

    /// Retrieves all active rule sets.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the query fails.
    async fn get_rule_sets(&self) -> AppResult<Vec<DuplicateRuleSet>>;

    /// Saves a new duplicate group.
    ///
    /// # Arguments
    /// * `group` - The duplicate group to save.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the insert fails.
    async fn save_group(&self, group: DuplicateGroup) -> AppResult<()>;

    /// Adds a candidate to a duplicate group.
    ///
    /// # Arguments
    /// * `candidate` - The duplicate candidate to save.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the insert fails.
    async fn save_candidate(&self, candidate: DuplicateCandidate) -> AppResult<()>;

    /// Retrieves groups based on their status.
    ///
    /// # Arguments
    /// * `status` - The status to filter by (e.g., "open").
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the query fails.
    async fn get_groups_by_status(&self, status: &str) -> AppResult<Vec<DuplicateGroup>>;

    /// Retrieves candidates for a specific group.
    ///
    /// # Arguments
    /// * `group_id` - The unique identifier of the duplicate group.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the query fails.
    async fn get_group_candidates(&self, group_id: &str) -> AppResult<Vec<DuplicateCandidate>>;

    /// Saves a resolution decision for a group.
    ///
    /// # Arguments
    /// * `resolution` - The resolution data to save.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the insert fails.
    async fn save_resolution(&self, resolution: DuplicateResolution) -> AppResult<()>;

    /// Updates the status of a duplicate group.
    ///
    /// # Arguments
    /// * `group_id` - The unique identifier of the duplicate group.
    /// * `status` - The new status.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the update fails.
    async fn update_group_status(&self, group_id: &str, status: &str) -> AppResult<()>;

    /// Runs a database-level scan to find exact duplicates and groups them.
    ///
    /// # Errors
    /// Returns `AppError::DatabaseError` if the query fails.
    async fn run_exact_match_scan(&self) -> AppResult<()>;
}
