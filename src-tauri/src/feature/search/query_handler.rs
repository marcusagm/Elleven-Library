use crate::core::error::AppResult;
use crate::core::models::{AssetSummaryDto, PageParams, SearchCriteria};
use crate::core::repository::AssetQueryHandler;
use std::sync::Arc;

/// Orchestrates advanced search operations.
pub struct SearchQueryHandler {
    /// The repository to delegate asset queries to.
    repository: Arc<dyn AssetQueryHandler>,
}

/// Implementation of the SearchQueryHandler.
impl SearchQueryHandler {
    /// Creates a new SearchQueryHandler.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository to delegate asset queries to.
    ///
    /// # Returns
    ///
    /// * `Self` - The new SearchQueryHandler.
    pub fn new(repository: Arc<dyn AssetQueryHandler>) -> Self {
        Self { repository }
    }

    /// Performs an advanced search based on the provided criteria.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The search criteria.
    /// * `page` - The pagination parameters.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<AssetSummaryDto>>` - The search results.
    pub async fn search(
        &self,
        criteria: SearchCriteria,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>> {
        self.repository.search_assets(criteria, page).await
    }
}
