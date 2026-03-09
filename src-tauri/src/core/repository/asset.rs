use crate::core::error::AppResult;
use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag};
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
    async fn list_folders(&self, parent_id: Option<String>) -> AppResult<Vec<Folder>>;

    /// Gets a folder by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the folder to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Folder>)` if the folder was found successfully.
    /// * `Err(AppResult<Folder>)` if the folder could not be found.
    async fn get_folder_by_id(&self, id: &str) -> AppResult<Option<Folder>>;

    /// Lists all unique tags.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Tag>)` if the tags were found successfully.
    /// * `Err(AppResult<Tag>)` if the tags could not be found.
    async fn list_tags(&self) -> AppResult<Vec<Tag>>;

    /// Performs an advanced search using complex criteria.
    ///
    /// # Arguments
    ///
    /// * `criteria` - The advanced search criteria.
    /// * `page` - The pagination parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
    /// * `Err(AppResult<AssetSummaryDto>)` if the search fails.
    async fn search_assets(
        &self,
        criteria: crate::core::models::SearchCriteria,
        page: PageParams,
    ) -> AppResult<Vec<AssetSummaryDto>>;

    /// Retrieves a list of asset IDs that are missing thumbnails.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of IDs to retrieve.
    async fn get_assets_needing_thumbnails(&self, limit: u32) -> AppResult<Vec<String>>;

    /// Retrieves a single asset by its unique ID.
    async fn get_asset_by_id(&self, id: &str) -> AppResult<Asset>;
}
