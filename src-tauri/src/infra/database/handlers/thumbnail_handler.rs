use crate::core::error::AppResult;
use crate::core::models::asset::{Asset, AssetState};
use crate::infra::database::ledger::SqliteAssetLedger;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

pub async fn handle_update_thumbnail(
    tx: &mut Transaction<'_, Sqlite>,
    asset_id: &str,
    thumbnail_path: &str,
) -> AppResult<Asset> {
    let now = Utc::now();
    let state_thumb = AssetState::Thumbnailed.to_string();

    // 1. Update Asset
    sqlx::query!(
        "UPDATE assets SET thumbnail_path = ?, state = ?, updated_at = ? WHERE id = ?",
        thumbnail_path,
        state_thumb,
        now,
        asset_id
    )
    .execute(&mut **tx)
    .await?;

    // 2. Audit Log
    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_THUMBNAIL",
        asset_id,
        serde_json::json!({ "path": thumbnail_path }),
        "COMPLETED",
        None,
    )
    .await?;

    // 3. Fetch and return
    SqliteAssetLedger::fetch_asset_by_id(tx, asset_id).await
}

pub async fn handle_regenerate_thumbnail(
    tx: &mut Transaction<'_, Sqlite>,
    asset_id: &str,
) -> AppResult<Asset> {
    let now = Utc::now();

    // 1. Clear thumbnail_path in assets table
    sqlx::query!(
        "UPDATE assets SET thumbnail_path = NULL, updated_at = ? WHERE id = ?",
        now,
        asset_id
    )
    .execute(&mut **tx)
    .await?;

    // 2. Audit Log
    SqliteAssetLedger::log_operation(
        tx,
        "REGENERATE_THUMBNAIL",
        asset_id,
        serde_json::json!({ "asset_id": asset_id }),
        "COMPLETED",
        None,
    )
    .await?;

    // 3. Fetch asset to return
    SqliteAssetLedger::fetch_asset_by_id(tx, asset_id).await
}
