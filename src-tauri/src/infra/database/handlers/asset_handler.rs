use crate::core::error::{AppError, AppResult};
use crate::core::ledger::command::CreateAssetPayload;
use crate::core::models::asset::Asset;
use crate::infra::database::ledger::SqliteAssetLedger;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use tracing::info;
use uuid::Uuid;

pub async fn handle_create(
    tx: &mut Transaction<'_, Sqlite>,
    payload: CreateAssetPayload,
) -> AppResult<Asset> {
    let asset_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let name = payload
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::ValidationFailed("Invalid file path".to_string()))?;
    let path_str = payload.path.to_string_lossy().to_string();
    let state_str = payload.state_init.to_string();
    let file_size_i64 = payload.file_size as i64;

    let created_at_val = payload.created_at.unwrap_or(now);
    let modified_at_val = payload.modified_at.unwrap_or(now);
    let added_at_val = now;

    // ─── V1 Signature-Based Move Recovery ────────────────────────
    // Before inserting, check if an existing asset with the same
    // file_size + created_at has a stale path (file no longer on disk).
    // If found, UPDATE that record instead of creating a duplicate.
    // This preserves tags, rating, notes, thumbnail, and colors.
    if let Some(filesystem_created_at) = payload.created_at {
        let move_candidates: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT id, folder_id, path FROM assets WHERE file_size = ? AND created_at = ?"
        )
        .bind(file_size_i64)
        .bind(filesystem_created_at)
        .fetch_all(&mut **tx)
        .await?;

        for (existing_asset_id, _old_folder_id, old_path_str) in &move_candidates {
            if !std::path::Path::new(old_path_str).exists() && old_path_str != &path_str {
                info!(
                    "Ledger: MOVE DETECTED (V1 recovery). Updating asset {} from '{}' to '{}'",
                    existing_asset_id, old_path_str, path_str
                );

                let folder_id_for_update = payload.folder_id.as_deref();
                sqlx::query!(
                    "UPDATE assets SET path = ?, name = ?, folder_id = ?, modified_at = ?, updated_at = ?, state = ? WHERE id = ?",
                    path_str,
                    name,
                    folder_id_for_update,
                    modified_at_val,
                    now,
                    state_str,
                    existing_asset_id
                )
                .execute(&mut **tx)
                .await?;

                SqliteAssetLedger::log_operation(
                    tx,
                    "CREATE_ASSET_MOVE_RECOVERY",
                    existing_asset_id,
                    serde_json::json!({
                        "old_path": old_path_str,
                        "new_path": path_str,
                        "recovery_type": "signature_match"
                    }),
                    "COMPLETED",
                    None,
                )
                .await?;

                return SqliteAssetLedger::fetch_asset_by_id(tx, existing_asset_id).await;
            }
        }
    }
    // ─── End Move Recovery ────────────────────────────────────────

    let path_ref = &path_str;
    let state_ref = &state_str;
    let format_type_ref = &payload.format_type;
    let family_ref = &payload.family;
    let folder_id_ref = payload.folder_id.as_deref();
    let asset_id_final_ref = &asset_id;

    let row = sqlx::query!(
        r#"
        INSERT INTO assets (
            id, name, path, state, format_type, family, file_size, 
            created_at, modified_at, added_at, updated_at, folder_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
            updated_at = excluded.updated_at
        RETURNING id as "id!"
        "#,
        asset_id_final_ref,
        name,
        path_ref,
        state_ref,
        format_type_ref,
        family_ref,
        file_size_i64,
        created_at_val,
        modified_at_val,
        added_at_val,
        now,
        folder_id_ref
    )
    .fetch_one(&mut **tx)
    .await?;

    let asset_id_final = row.id.to_string();

    // 2. Audit Log
    let op_payload = serde_json::to_value(&payload).map_err(|e| {
        AppError::Internal(format!("Failed to serialize payload: {}", e))
    })?;

    SqliteAssetLedger::log_operation(
        tx,
        "CREATE_ASSET",
        &asset_id_final,
        op_payload,
        "COMPLETED",
        None,
    )
    .await?;

    let asset = Asset {
        id: asset_id_final,
        name: name.to_string(),
        path: payload.path.clone(),
        state: payload.state_init,
        format_type: payload.format_type.clone(),
        family: payload.family.clone(),
        file_size: payload.file_size,
        created_at: Some(created_at_val),
        modified_at: Some(modified_at_val),
        added_at: Some(added_at_val),
        updated_at: Some(now),
        width: None,
        height: None,
        duration_secs: None,
        technical_payload: None,
        semantic_payload: None,
        dominant_color: None,
        folder_id: payload.folder_id.clone(),
        thumbnail_path: None,
        rating: None,
        notes: None,
    };

    Ok(asset)
}

pub async fn handle_batch_create(
    tx: &mut Transaction<'_, Sqlite>,
    payloads: Vec<CreateAssetPayload>,
) -> AppResult<Asset> {
    let mut created_assets = Vec::new();

    for payload in payloads {
        let asset_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let name = payload
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                AppError::ValidationFailed("Invalid file path".to_string())
            })?;
        let state_str = payload.state_init.to_string();
        let path_str = payload.path.to_string_lossy().to_string();
        let file_size_i64 = payload.file_size as i64;

        let created_at_val = payload.created_at.unwrap_or(now);
        let modified_at_val = payload.modified_at.unwrap_or(now);
        let added_at_val = now;

        let path_ref = &path_str;
        let state_ref = &state_str;
        let format_type_ref = &payload.format_type;
        let family_ref = &payload.family;
        let folder_id_ref = payload.folder_id.as_deref();
        let asset_id_ref = &asset_id;

        // 1. Insert Asset (Upsert)
        let row = sqlx::query!(
            r#"
            INSERT INTO assets (
                id, name, path, state, format_type, family, file_size, 
                created_at, modified_at, added_at, updated_at, folder_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
                updated_at = excluded.updated_at
            RETURNING id as "id!"
            "#,
            asset_id_ref,
            name,
            path_ref,
            state_ref,
            format_type_ref,
            family_ref,
            file_size_i64,
            created_at_val,
            modified_at_val,
            added_at_val,
            now,
            folder_id_ref
        )
        .fetch_one(&mut **tx)
        .await?;

        let asset_id = row.id;

        // 2. Audit Log
        let op_payload = serde_json::to_value(&payload).map_err(|e| {
            AppError::Internal(format!("Failed to serialize payload: {}", e))
        })?;

        SqliteAssetLedger::log_operation(
            tx,
            "CREATE_ASSET_BATCH_MEMBER",
            &asset_id,
            op_payload,
            "COMPLETED",
            None,
        )
        .await?;

        created_assets.push(Asset {
            id: asset_id.clone(),
            name: name.to_string(),
            path: payload.path.clone(),
            state: payload.state_init,
            format_type: payload.format_type.clone(),
            family: payload.family.clone(),
            file_size: payload.file_size,
            created_at: Some(created_at_val),
            modified_at: Some(modified_at_val),
            added_at: Some(added_at_val),
            updated_at: Some(now),
            width: None,
            height: None,
            duration_secs: None,
            technical_payload: None,
            semantic_payload: None,
            dominant_color: None,
            folder_id: payload.folder_id.clone(),
            thumbnail_path: None,
            rating: None,
            notes: None,
        });
    }

    created_assets
        .into_iter()
        .next()
        .ok_or_else(|| AppError::ValidationFailed("Empty batch".to_string()))
}
