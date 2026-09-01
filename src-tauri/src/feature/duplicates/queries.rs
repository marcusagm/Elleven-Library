use crate::core::models::{DuplicateCandidate, DuplicateGroup};
use crate::core::repository::DuplicatesRepository;
use crate::core::error::AppResult;
use std::sync::Arc;

/// Service handler for duplicates-related queries.
pub struct DuplicateQueryService {
    repository: Arc<dyn DuplicatesRepository>,
}

impl DuplicateQueryService {
    /// Creates a new instance of DuplicateQueryService.
    ///
    /// # Arguments
    /// * `repository` - The duplicate repository port.
    pub fn new(repository: Arc<dyn DuplicatesRepository>) -> Self {
        Self { repository }
    }

    /// Retrieves duplicate groups filtered by status.
    ///
    /// # Arguments
    /// * `status` - The group status (e.g., "open", "resolved").
    ///
    /// # Errors
    /// Returns `AppError::Database` if the query fails.
    pub async fn get_groups_by_status(&self, status: &str) -> AppResult<Vec<DuplicateGroup>> {
        self.repository.get_groups_by_status(status).await
    }

    /// Retrieves all candidates for a given group.
    ///
    /// # Arguments
    /// * `group_id` - The unique identifier of the duplicate group.
    ///
    /// # Errors
    /// Returns `AppError::Database` if the query fails.
    pub async fn get_group_candidates(&self, group_id: &str) -> AppResult<Vec<DuplicateCandidate>> {
        self.repository.get_group_candidates(group_id).await
    }
}
