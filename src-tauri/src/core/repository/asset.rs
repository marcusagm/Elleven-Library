use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, PageParams};
use async_trait::async_trait;

/// Port for read-only asset operations.
#[async_trait]
pub trait AssetQueryHandler: Send + Sync {
    /// Retrieves a list of all assets in the library.
    /// Obsolete: use list_paginated for better performance.
    ///
    /// # Arguments
    ///
    /// * `self` - The asset query handler.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Asset>)` if the assets were found successfully.
    /// * `Err(AppResult<Asset>)` if the assets could not be found.
    async fn find_all(&self) -> AppResult<Vec<Asset>>;

    /// Retrieves a single asset by ID, including its full metadata join.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the asset to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Asset>)` if the asset was found successfully.
    /// * `Err(AppResult<Asset>)` if the asset could not be found.
    async fn get_by_id(&self, id: &str) -> AppResult<Option<Asset>>;

    /// Returns a paginated list of assets focused on performance.
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
    async fn list_paginated(
        &self,
        filter: AssetFilter,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>>;
}
