use crate::core::models::{Asset, AssetFilter, AssetSummaryDto, PageParams};
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
pub async fn get_assets_v2(
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
pub async fn get_asset_v2(
    service: State<'_, AssetQueryService>,
    id: String,
) -> Result<Option<Asset>, String> {
    service.get_asset(&id).await.map_err(|e| e.to_string())
}
