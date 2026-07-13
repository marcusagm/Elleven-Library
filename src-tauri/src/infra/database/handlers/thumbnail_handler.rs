//! Thumbnail command handlers.
//!
//! Encapsulates SQL mutations for persisting and invalidating asset thumbnail paths,
//! decoupled from the actual image generation workers.
use crate::core::error::AppResult;
use crate::core::models::asset::{Asset, AssetState};
use super::shared;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

/// Persists a generated thumbnail path for an asset and transitions its state to `Thumbnailed`.
///
/// # Arguments
///
/// * `tx` - The active database transaction.
/// * `asset_id` - The unique identifier of the asset.
/// * `thumbnail_path` - The filesystem path to the generated thumbnail.
///
/// # Errors
///
/// Returns `AppError` if the asset record does not exist or the database update fails.
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
    shared::log_operation(
        tx,
        "UPDATE_THUMBNAIL",
        asset_id,
        serde_json::json!({ "path": thumbnail_path }),
        "COMPLETED",
        None,
    )
    .await?;

    shared::fetch_asset_by_id(tx, asset_id).await
}

/// Clears an asset's cached thumbnail path, signalling that regeneration is required.
///
/// The actual thumbnail generation is triggered by the `ThumbnailInvalidated` domain event
/// emitted by the Ledger after this handler commits.
///
/// # Arguments
///
/// * `tx` - The active database transaction.
/// * `asset_id` - The unique identifier of the asset.
///
/// # Errors
///
/// Returns `AppError` if the asset record does not exist or the database update fails.
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
    shared::log_operation(
        tx,
        "REGENERATE_THUMBNAIL",
        asset_id,
        serde_json::json!({ "asset_id": asset_id }),
        "COMPLETED",
        None,
    )
    .await?;

    shared::fetch_asset_by_id(tx, asset_id).await
}
