use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag, SmartFolder, LibraryStats};
use crate::feature::assets::queries::AssetQueryService;
use tauri::State;

use crate::core::error::AppResult;

/// RPC Command to list assets with filters and pagination.
///
/// # Arguments
///
/// * `service` - The asset query service.
/// * `filter` - The filter to apply to the assets.
/// * `page` - The pagination parameters.
///
/// # Returns
///
/// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
/// * `Err(AppError)` if the assets could not be found.
#[tauri::command]
pub async fn get_assets(
    service: State<'_, AssetQueryService>,
    filter: AssetFilter,
    page: PageParams,
) -> AppResult<Vec<AssetSummaryDto>> {
    service.list_assets(filter, page).await
}

/// RPC Command to get a single asset with full metadata.
///
/// # Arguments
///
/// * `service` - The asset query service.
/// * `id` - The ID of the asset to retrieve.
///
/// # Returns
///
/// * `Ok(Option<Asset>)` if the asset was found successfully.
/// * `Err(AppError)` if the asset could not be found.
#[tauri::command]
pub async fn get_asset(
    service: State<'_, AssetQueryService>,
    id: String,
) -> AppResult<Option<Asset>> {
    service.get_asset(&id).await
}

/// RPC Command to list folders under a parent.
///
/// # Arguments
///
/// * `service` - The asset query service.
/// * `parent_id` - The ID of the parent folder.
///
/// # Returns
///
/// * `Ok(Vec<Folder>)` if the folders were found successfully.
/// * `Err(AppError)` if the folders could not be found.
#[tauri::command]
pub async fn list_folders(
    service: State<'_, AssetQueryService>,
    parent_id: Option<String>,
) -> AppResult<Vec<Folder>> {
    service.list_folders(parent_id).await
}

/// RPC Command to list all tags.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(Vec<Tag>)` if the tags were found successfully.
/// * `Err(AppError)` if the tags could not be found.
#[tauri::command]
pub async fn list_tags(service: State<'_, AssetQueryService>) -> AppResult<Vec<Tag>> {
    service.list_tags().await
}

/// RPC Command to perform an advanced search.
///
/// # Arguments
///
/// * `service` - The search query handler.
/// * `criteria` - The advanced search criteria.
/// * `page` - The pagination parameters.
///
/// # Returns
///
/// * `Ok(Vec<AssetSummaryDto>)` if the assets were found successfully.
/// * `Err(AppError)` if the search fails.
#[tauri::command]
pub async fn search_assets(
    service: State<'_, crate::feature::search::SearchQueryHandler>,
    criteria: crate::core::models::SearchCriteria,
    page: PageParams,
) -> AppResult<Vec<AssetSummaryDto>> {
    service.search(criteria, page).await
}

/// RPC Command to get all tags associated with a specific asset.
///
/// # Arguments
///
/// * `service` - The asset query service.
/// * `asset_id` - The unique identifier of the asset.
///
/// # Returns
///
/// * `Ok(Vec<Tag>)` if the tags were found successfully.
/// * `Err(AppError)` if the query fails.
#[tauri::command]
pub async fn get_tags_for_asset(
    service: State<'_, AssetQueryService>,
    asset_id: String,
) -> AppResult<Vec<Tag>> {
    service.get_tags_for_asset(&asset_id).await
}

/// RPC Command to get all subfolders.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(Vec<Folder>)` if the folders were found successfully.
/// * `Err(AppError)` if the folders could not be found.
#[tauri::command]
pub async fn get_all_subfolders(service: State<'_, AssetQueryService>) -> AppResult<Vec<Folder>> {
    service.list_all_subfolders().await
}

/// RPC Command to get subfolder asset counts.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(Vec<(String, i64)>)` if the counts were found successfully.
/// * `Err(AppError)` if the counts could not be found.
#[tauri::command]
pub async fn get_subfolder_counts(
    service: State<'_, AssetQueryService>,
) -> AppResult<Vec<(String, i64)>> {
    service.get_subfolder_asset_counts().await
}

/// RPC Command to get root location counts.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(Vec<(String, i64)>)` if the counts were found successfully.
/// * `Err(AppError)` if the counts could not be found.
#[tauri::command]
pub async fn get_location_root_counts(
    service: State<'_, AssetQueryService>,
) -> AppResult<Vec<(String, i64)>> {
    service.get_location_root_counts().await
}

/// RPC Command to get all smart folders.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(Vec<SmartFolder>)` if the smart folders were found successfully.
/// * `Err(AppError)` if the query fails.
#[tauri::command]
pub async fn get_smart_folders(
    service: State<'_, AssetQueryService>,
) -> AppResult<Vec<SmartFolder>> {
    service.list_smart_folders().await
}

/// RPC Command to get the total number of assets matching a filter.
///
/// # Arguments
///
/// * `service` - The asset query service.
/// * `filter` - The asset filter parameters.
///
/// # Returns
///
/// * `Ok(i64)` count of assets.
/// * `Err(AppError)` if the query fails.
#[tauri::command]
pub async fn get_asset_count_filtered(
    service: State<'_, AssetQueryService>,
    filter: AssetFilter,
) -> AppResult<i64> {
    service.get_asset_count(filter).await
}

/// RPC Command to get library wide statistics.
///
/// # Arguments
///
/// * `service` - The asset query service.
///
/// # Returns
///
/// * `Ok(LibraryStats)` object containing aggregates.
/// * `Err(AppError)` if the query fails.
#[tauri::command]
pub async fn get_library_stats(
    service: State<'_, AssetQueryService>,
) -> AppResult<LibraryStats> {
    service.get_library_stats().await
}
