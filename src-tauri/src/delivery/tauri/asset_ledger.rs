use crate::core::ledger::command::{CreateFolderPayload, LedgerCommand, UpdateTagsPayload};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::Asset;
use std::sync::Arc;
use tauri::State;

/// RPC Command to create a new logical folder.
#[tauri::command]
pub async fn create_folder(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    payload: CreateFolderPayload,
) -> Result<Asset, String> {
    ledger
        .execute(LedgerCommand::CreateFolder(payload))
        .await
        .map_err(|e| e.to_string())
}

/// RPC Command to move an asset to a folder.
#[tauri::command]
pub async fn set_asset_folder(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    asset_id: String,
    folder_id: Option<String>,
) -> Result<Asset, String> {
    ledger
        .execute(LedgerCommand::SetAssetFolder {
            asset_id,
            folder_id,
        })
        .await
        .map_err(|e| e.to_string())
}

/// RPC Command to update asset tags.
#[tauri::command]
pub async fn update_asset_tags(
    ledger: State<'_, Arc<dyn TransactionalAssetLedger>>,
    payload: UpdateTagsPayload,
) -> Result<Asset, String> {
    ledger
        .execute(LedgerCommand::UpdateTags(payload))
        .await
        .map_err(|e| e.to_string())
}
