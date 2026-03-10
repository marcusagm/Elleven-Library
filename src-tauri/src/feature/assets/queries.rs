use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag};
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

    /// Lists folders under a parent.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The ID of the parent folder.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Folder>)` if the folders were found successfully.
    /// * `Err(AppResult<Folder>)` if the folders could not be found.
    pub async fn list_folders(&self, parent_id: Option<String>) -> AppResult<Vec<Folder>> {
        self.repository.list_folders(parent_id).await
    }

    /// Lists all tags.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Tag>)` if the tags were found successfully.
    /// * `Err(AppResult<Tag>)` if the tags could not be found.
    pub async fn list_tags(&self) -> AppResult<Vec<Tag>> {
        self.repository.list_tags().await
    }

    /// Gets all tags associated with a specific asset.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The unique identifier of the asset.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Tag>)` if the tags were found successfully.
    /// * `Err(AppError)` if the query fails.
    pub async fn get_tags_for_asset(&self, asset_id: &str) -> AppResult<Vec<Tag>> {
        self.repository.get_tags_for_asset(asset_id).await
    }
}
