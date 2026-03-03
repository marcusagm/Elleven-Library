use crate::db::models::{AssetMetadata, LibraryStats, Tag};
use crate::db::Db;
use crate::error::AppResult;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn create_tag(
    db: State<'_, Arc<Db>>,
    name: String,
    parent_id: Option<i64>,
    color: Option<String>,
) -> AppResult<i64> {
    Ok(db.create_tag(&name, parent_id, color).await?)
}

#[tauri::command]
pub async fn update_tag(
    db: State<'_, Arc<Db>>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
    parent_id: Option<i64>,
    order_index: Option<i64>,
) -> AppResult<()> {
    Ok(db
        .update_tag(id, name, color, parent_id, order_index)
        .await?)
}

#[tauri::command]
pub async fn delete_tag(db: State<'_, Arc<Db>>, id: i64) -> AppResult<()> {
    Ok(db.delete_tag(id).await?)
}

#[tauri::command]
pub async fn get_all_tags(db: State<'_, Arc<Db>>) -> AppResult<Vec<Tag>> {
    Ok(db.get_all_tags().await?)
}

#[tauri::command]
pub async fn get_library_stats(db: State<'_, Arc<Db>>) -> AppResult<LibraryStats> {
    Ok(db.get_library_stats().await?)
}

#[tauri::command]
pub async fn add_tag_to_asset(db: State<'_, Arc<Db>>, asset_id: i64, tag_id: i64) -> AppResult<()> {
    Ok(db.add_tag_to_asset(asset_id, tag_id).await?)
}

#[tauri::command]
pub async fn remove_tag_from_asset(
    db: State<'_, Arc<Db>>,
    asset_id: i64,
    tag_id: i64,
) -> AppResult<()> {
    Ok(db.remove_tag_from_asset(asset_id, tag_id).await?)
}

#[tauri::command]
pub async fn get_tags_for_asset(db: State<'_, Arc<Db>>, asset_id: i64) -> AppResult<Vec<Tag>> {
    Ok(db.get_tags_for_asset(asset_id).await?)
}

#[tauri::command]
pub async fn add_tags_to_assets_batch(
    db: State<'_, Arc<Db>>,
    asset_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    Ok(db.add_tags_to_assets_batch(asset_ids, tag_ids).await?)
}

/// Retrieves a filtered and paginated list of assets.
///
/// This acts as a wrapper for the `Db::get_assets_filtered` method, enabling frontend filtering,
/// including tag matching, folder recursion, advanced queries, and fuzzy text search.
///
/// # Errors
/// Returns `AppResult::Err` if the database query fails or the connection is lost.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn get_assets_filtered(
    db: State<'_, Arc<Db>>,
    limit: i32,
    offset: i32,
    tag_ids: Vec<i64>,
    match_all: bool,
    untagged: Option<bool>,
    folder_id: Option<i64>,
    recursive: bool,
    sort_by: Option<String>,
    sort_order: Option<String>,
    advanced_query: Option<String>,
    search_query: Option<String>,
    search_fuzzy: Option<bool>,
) -> AppResult<Vec<AssetMetadata>> {
    Ok(db
        .get_assets_filtered(
            limit,
            offset,
            tag_ids,
            match_all,
            untagged,
            folder_id,
            recursive,
            sort_by,
            sort_order,
            advanced_query,
            search_query,
            search_fuzzy,
        )
        .await?)
}

/// Gets the total count of assets matching the criteria.
///
/// Acts as a wrapper for `Db::get_asset_count_filtered`. Useful for frontend pagination lengths.
///
/// # Errors
/// Returns `AppResult::Err` if the database query fails.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn get_asset_count_filtered(
    db: State<'_, Arc<Db>>,
    tag_ids: Vec<i64>,
    match_all: bool,
    untagged: Option<bool>,
    folder_id: Option<i64>,
    recursive: bool,
    advanced_query: Option<String>,
    search_query: Option<String>,
    search_fuzzy: Option<bool>,
) -> AppResult<i64> {
    Ok(db
        .get_asset_count_filtered(
            tag_ids,
            match_all,
            untagged,
            folder_id,
            recursive,
            advanced_query,
            search_query,
            search_fuzzy,
        )
        .await?)
}

#[tauri::command]
pub async fn update_asset_rating(db: State<'_, Arc<Db>>, id: i64, rating: i32) -> AppResult<()> {
    Ok(db.update_asset_rating(id, rating).await?)
}

#[tauri::command]
pub async fn update_asset_notes(db: State<'_, Arc<Db>>, id: i64, notes: String) -> AppResult<()> {
    Ok(db.update_asset_notes(id, notes).await?)
}
#[tauri::command]
pub async fn remove_tags_from_assets_batch(
    db: State<'_, Arc<Db>>,
    asset_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    Ok(db.remove_tags_from_assets_batch(asset_ids, tag_ids).await?)
}

#[tauri::command]
pub async fn replace_tags_for_assets_batch(
    db: State<'_, Arc<Db>>,
    asset_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    Ok(db.replace_tags_for_assets_batch(asset_ids, tag_ids).await?)
}
