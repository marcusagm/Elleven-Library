use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, Folder, PageParams, Tag};
use crate::feature::assets::queries::AssetQueryService;
use tauri::State;

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
/// * `Err(String)` if the assets could not be found.
#[tauri::command]
pub async fn get_assets(
    service: State<'_, AssetQueryService>,
    filter: AssetFilter,
    page: PageParams,
) -> Result<Vec<AssetSummaryDto>, String> {
    service
        .list_assets(filter, page)
        .await
        .map_err(|e| e.to_string())
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
/// * `Err(String)` if the asset could not be found.
#[tauri::command]
pub async fn get_asset(
    service: State<'_, AssetQueryService>,
    id: String,
) -> Result<Option<Asset>, String> {
    service.get_asset(&id).await.map_err(|e| e.to_string())
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
/// * `Err(String)` if the folders could not be found.
#[tauri::command]
pub async fn list_folders(
    service: State<'_, AssetQueryService>,
    parent_id: Option<String>,
) -> Result<Vec<Folder>, String> {
    service
        .list_folders(parent_id)
        .await
        .map_err(|e| e.to_string())
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
/// * `Err(String)` if the tags could not be found.
#[tauri::command]
pub async fn list_tags(service: State<'_, AssetQueryService>) -> Result<Vec<Tag>, String> {
    service.list_tags().await.map_err(|e| e.to_string())
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
/// * `Err(String)` if the search fails.
#[tauri::command]
pub async fn search_assets(
    service: State<'_, crate::feature::search::SearchQueryHandler>,
    criteria: crate::core::models::SearchCriteria,
    page: PageParams,
) -> Result<Vec<AssetSummaryDto>, String> {
    service
        .search(criteria, page)
        .await
        .map_err(|e| e.to_string())
}
