use crate::core::ledger::command::{CreateFolderPayload, LedgerCommand, UpdateTagsPayload};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::Asset;
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
