use crate::core::error::{AppError, AppResult};
use crate::core::ledger::command::{
    UpdateAssetColorsPayload, UpdateAssetNotesPayload, UpdateAssetRatingPayload,
    UpdateTechnicalMetadataPayload,
};
use crate::core::models::asset::Asset;
use crate::infra::database::ledger::SqliteAssetLedger;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

pub async fn handle_update_asset_colors(
    tx: &mut Transaction<'_, Sqlite>,
    payload: UpdateAssetColorsPayload,
) -> AppResult<Asset> {
    let now = Utc::now();
    let asset_id_ref = &payload.asset_id;

    // 1. Delete existing colors for this asset
    sqlx::query!("DELETE FROM asset_colors WHERE asset_id = ?", asset_id_ref)
        .execute(&mut **tx)
        .await?;

    // 2. Insert new colors
    for color in &payload.colors {
        sqlx::query!(
            r#"
            INSERT INTO asset_colors (asset_id, hex_color, lab_lightness, lab_green_red, lab_blue_yellow, percentage, rank)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            asset_id_ref,
            color.hex_color,
            color.lab_lightness,
            color.lab_green_red,
            color.lab_blue_yellow,
            color.percentage,
            color.rank
        )
        .execute(&mut **tx)
        .await?;
    }

    // 3. Update dominant_color in assets table if we have colors
    if let Some(dominant) = payload.colors.first() {
        sqlx::query(
            "UPDATE assets SET dominant_color = ?, updated_at = ? WHERE id = ?",
        )
        .bind(serde_json::json!(dominant.hex_color))
        .bind(now)
        .bind(asset_id_ref)
        .execute(&mut **tx)
        .await?;
    }

    // 4. Audit Log
    let op_payload = serde_json::to_value(&payload).map_err(|e| {
        AppError::Internal(format!("Failed to serialize payload: {}", e))
    })?;

    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_ASSET_COLORS",
        asset_id_ref,
        op_payload,
        "COMPLETED",
        None,
    )
    .await?;

    SqliteAssetLedger::fetch_asset_by_id(tx, &payload.asset_id).await
}

pub async fn handle_update_rating(
    tx: &mut Transaction<'_, Sqlite>,
    payload: UpdateAssetRatingPayload,
) -> AppResult<Asset> {
    let now = Utc::now();
    sqlx::query!(
        "UPDATE assets SET rating = ?, updated_at = ? WHERE id = ?",
        payload.rating,
        now,
        payload.asset_id
    )
    .execute(&mut **tx)
    .await?;

    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_ASSET_RATING",
        &payload.asset_id,
        serde_json::json!({ "rating": payload.rating }),
        "COMPLETED",
        None,
    )
    .await?;

    SqliteAssetLedger::fetch_asset_by_id(tx, &payload.asset_id).await
}

pub async fn handle_update_notes(
    tx: &mut Transaction<'_, Sqlite>,
    payload: UpdateAssetNotesPayload,
) -> AppResult<Asset> {
    let now = Utc::now();
    sqlx::query!(
        "UPDATE assets SET notes = ?, updated_at = ? WHERE id = ?",
        payload.notes,
        now,
        payload.asset_id
    )
    .execute(&mut **tx)
    .await?;

    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_ASSET_NOTES",
        &payload.asset_id,
        serde_json::json!({ "notes": payload.notes }),
        "COMPLETED",
        None,
    )
    .await?;

    SqliteAssetLedger::fetch_asset_by_id(tx, &payload.asset_id).await
}

pub async fn handle_update_format(
    tx: &mut Transaction<'_, Sqlite>,
    asset_id: &str,
    format: &str,
) -> AppResult<Asset> {
    let now = Utc::now();
    sqlx::query!(
        "UPDATE assets SET format_type = ?, updated_at = ? WHERE id = ?",
        format,
        now,
        asset_id
    )
    .execute(&mut **tx)
    .await?;

    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_FORMAT",
        asset_id,
        serde_json::json!({ "format": format }),
        "COMPLETED",
        None,
    )
    .await?;

    SqliteAssetLedger::fetch_asset_by_id(tx, asset_id).await
}

pub async fn handle_update_technical_metadata(
    tx: &mut Transaction<'_, Sqlite>,
    payload: UpdateTechnicalMetadataPayload,
) -> AppResult<Asset> {
    let now = Utc::now();

    sqlx::query!(
        r#"
        INSERT INTO asset_metadata_envelope (
            asset_id, width, height, duration_secs, 
            technical_payload, semantic_payload, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(asset_id) DO UPDATE SET
            width = excluded.width,
            height = excluded.height,
            duration_secs = excluded.duration_secs,
            technical_payload = excluded.technical_payload,
            semantic_payload = excluded.semantic_payload,
            updated_at = excluded.updated_at
        "#,
        payload.asset_id,
        payload.width,
        payload.height,
        payload.duration_secs,
        payload.technical_payload,
        payload.semantic_payload,
        now,
        now
    )
    .execute(&mut **tx)
    .await?;

    // Update assets.updated_at to reflect metadata change
    sqlx::query!(
        "UPDATE assets SET updated_at = ? WHERE id = ?",
        now,
        payload.asset_id
    )
    .execute(&mut **tx)
    .await?;

    SqliteAssetLedger::log_operation(
        tx,
        "UPDATE_TECHNICAL_METADATA",
        &payload.asset_id,
        serde_json::to_value(&payload).unwrap_or_default(),
        "COMPLETED",
        None,
    )
    .await?;

    SqliteAssetLedger::fetch_asset_by_id(tx, &payload.asset_id).await
}
