use crate::core::ledger::command::{
    BatchTagsPayload, CreateFolderPayload, CreateTagPayload, LedgerCommand, UpdateTagPayload,
    UpdateTagsPayload,
};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::{Asset, Tag};
use crate::feature::assets::queries::AssetQueryService;
use crate::feature::library::indexer::LibraryIndexer;
use crate::processing::watcher::WatcherService;
use std::sync::Arc;
use tauri::State;

use crate::core::error::AppResult;

/// RPC Command to create a new logical folder.
///
/// # Arguments
///
/// * `ledger` - The asset ledger.
/// * `payload` - The folder payload.
///
/// # Returns
///
/// The created folder.
#[tauri::command]
pub async fn create_folder(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    payload: CreateFolderPayload,
) -> AppResult<Asset> {
    ledger.execute(LedgerCommand::CreateFolder(payload)).await
}

/// RPC Command to move an asset to a folder.
///
/// # Arguments
///
/// * `ledger` - The asset ledger.
/// * `asset_id` - The asset ID.
/// * `folder_id` - The folder ID.
///
/// # Returns
///
/// The updated asset.
#[tauri::command]
pub async fn set_asset_folder(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    asset_id: String,
    folder_id: Option<String>,
) -> AppResult<Asset> {
    ledger
        .execute(LedgerCommand::SetAssetFolder {
            asset_id,
            folder_id,
        })
        .await
}

/// RPC Command to update asset tags.
///
/// # Arguments
///
/// * `ledger` - The asset ledger.
/// * `payload` - The tag payload.
///
/// # Returns
///
/// The updated asset.
#[tauri::command]
pub async fn update_asset_tags(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    payload: UpdateTagsPayload,
) -> AppResult<Asset> {
    ledger.execute(LedgerCommand::UpdateTags(payload)).await
}

/// RPC Command to create a new taxonomy tag.
///
/// Creates the tag via the Ledger (mutation) and then queries it back
/// to return the full Tag entity with generated UUID.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `service` - The asset query service for reading back the created tag.
/// * `name` - The display name of the tag (must be unique).
/// * `parent_id` - Optional parent tag ID for hierarchical organization.
/// * `color` - Optional hex color code for the tag.
///
/// # Errors
///
/// Returns `AppError` if the tag name is not unique or the DB operation fails.
#[tauri::command]
pub async fn create_tag(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    service: State<'_, AssetQueryService>,
    name: String,
    parent_id: Option<String>,
    color: Option<String>,
) -> AppResult<Tag> {
    let result = ledger
        .execute(LedgerCommand::CreateTag(CreateTagPayload {
            name,
            parent_id,
            color,
        }))
        .await?;

    // The Ledger returns a dummy Asset with the tag_id in the id field.
    // Query the actual tag from the database to return the real Tag entity.
    let all_tags = service.list_tags().await?;
    let created_tag = all_tags
        .into_iter()
        .find(|tag| tag.id == result.id)
        .ok_or_else(|| {
            crate::core::error::AppError::Internal(
                "Tag was created but could not be read back".to_string(),
            )
        })?;

    Ok(created_tag)
}

/// RPC Command to update an existing tag's properties.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `id` - The unique identifier of the tag.
/// * `name` - Optional new display name.
/// * `color` - Optional new hex color code.
/// * `parent_id` - Optional new parent tag ID.
/// * `order_index` - Optional new sorting order index.
///
/// # Errors
///
/// Returns `AppError` if the tag doesn't exist or the DB operation fails.
#[tauri::command]
pub async fn update_tag(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    id: String,
    name: Option<String>,
    color: Option<String>,
    parent_id: Option<String>,
    order_index: Option<i64>,
) -> AppResult<()> {
    ledger
        .execute(LedgerCommand::UpdateTag(UpdateTagPayload {
            id,
            name,
            color,
            parent_id,
            order_index,
        }))
        .await?;
    Ok(())
}

/// RPC Command to delete a tag, removing it from all assets.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `id` - The unique identifier of the tag to delete.
///
/// # Errors
///
/// Returns `AppError` if the DB operation fails.
#[tauri::command]
pub async fn delete_tag(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    id: String,
) -> AppResult<()> {
    ledger.execute(LedgerCommand::DeleteTag { id }).await?;
    Ok(())
}

/// RPC Command to associate multiple tags with multiple assets in a single transaction.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `asset_ids` - The asset IDs to tag.
/// * `tag_ids` - The tag IDs to apply.
///
/// # Errors
///
/// Returns `AppError` if the DB operation fails.
#[tauri::command]
pub async fn add_tags_to_assets_batch(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    asset_ids: Vec<String>,
    tag_ids: Vec<String>,
) -> AppResult<()> {
    ledger
        .execute(LedgerCommand::AddTagsToAssetsBatch(BatchTagsPayload {
            asset_ids,
            tag_ids,
        }))
        .await?;
    Ok(())
}

/// RPC Command to remove specific tags from multiple assets in a single transaction.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `asset_ids` - The asset IDs to untag.
/// * `tag_ids` - The tag IDs to remove.
///
/// # Errors
///
/// Returns `AppError` if the DB operation fails.
#[tauri::command]
pub async fn remove_tags_from_assets_batch(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    asset_ids: Vec<String>,
    tag_ids: Vec<String>,
) -> AppResult<()> {
    ledger
        .execute(LedgerCommand::RemoveTagsFromAssetsBatch(BatchTagsPayload {
            asset_ids,
            tag_ids,
        }))
        .await?;
    Ok(())
}

/// RPC Command to replace all tags for multiple assets with a new set.
///
/// # Arguments
///
/// * `ledger` - The transactional asset ledger.
/// * `asset_ids` - The asset IDs to update.
/// * `tag_ids` - The new set of tag IDs.
///
/// # Errors
///
/// Returns `AppError` if the DB operation fails.
#[tauri::command]
pub async fn replace_tags_for_assets_batch(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    asset_ids: Vec<String>,
    tag_ids: Vec<String>,
) -> AppResult<()> {
    ledger
        .execute(LedgerCommand::ReplaceTagsForAssetsBatch(BatchTagsPayload {
            asset_ids,
            tag_ids,
        }))
        .await?;
    Ok(())
}

/// RPC Command to remove a location/folder.
///
/// # Arguments
///
/// * `folder_id` - The folder ID to remove.
///
/// # Errors
///
/// Returns `AppError` if the DB operation fails.
#[tauri::command]
pub async fn remove_location(
    folder_id: String,
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    queries: State<'_, AssetQueryService>,
    watcher: State<'_, Arc<WatcherService>>,
) -> AppResult<()> {
    // 1. Get thumbnails before delete
    let thumbnails = queries
        .get_folder_thumbnails(&folder_id)
        .await
        .unwrap_or_default();

    // 2. Ledger operation
    let result = ledger
        .execute(LedgerCommand::RemoveFolder(
            crate::core::ledger::command::RemoveFolderPayload {
                folder_id: folder_id.clone(),
            },
        ))
        .await?;

    // 3. Cleanup Watcher
    let path = result.path;
    let _ = watcher.unwatch(path).await;

    // 4. Cleanup thumbnails
    for thumb in thumbnails {
        let _ = tokio::fs::remove_file(&thumb).await;
    }

    Ok(())
}

/// RPC Command to manually start indexing a location.
///
/// # Arguments
///
/// * `path` - The path to index.
/// * `indexer` - The library indexer.
///
/// # Errors
///
/// Returns `AppError` if the indexing fails.
#[tauri::command]
pub async fn start_indexing(
    path: String,
    indexer: State<'_, Arc<LibraryIndexer>>,
) -> AppResult<()> {
    let indexer_ref = indexer.inner().clone();
    let path_buf = std::path::PathBuf::from(path);
    tauri::async_runtime::spawn(async move {
        let _ = indexer_ref.scan_directory(path_buf).await;
    });
    Ok(())
}
