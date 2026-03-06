use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, PageParams};
use crate::core::repository::AssetQueryHandler;
use std::sync::Arc;

/// Service handler for asset-related queries.
pub struct AssetQueryService {
    repository: Arc<dyn AssetQueryHandler>,
}

/// Implementation of the AssetQueryService struct.
impl AssetQueryService {
    /// Creates a new instance of AssetQueryService.
    ///
    /// # Arguments
    ///
    /// * `repository` - The asset query repository.
    ///
    /// # Returns
    ///
    /// * `AssetQueryService` - The asset query service.
    pub fn new(repository: Arc<dyn AssetQueryHandler>) -> Self {
        Self { repository }
    }

    /// Lists assets with specific filters and pagination.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter to apply to the assets.
    /// * `page` - The pagination parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
    /// * `Err(AppResult<AssetSummaryDto>)` if the assets could not be found.
    pub async fn list_assets(
        &self,
        filter: AssetFilter,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>> {
        self.repository.list_paginated(filter, page).await
    }

    /// Gets a single asset with full metadata.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Asset>)` if the asset was found successfully.
    /// * `Err(AppResult<Asset>)` if the asset could not be found.
    pub async fn get_asset(&self, id: &str) -> AppResult<Option<Asset>> {
        self.repository.get_by_id(id).await
    }
}
