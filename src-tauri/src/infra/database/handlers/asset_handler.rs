//! Asset lifecycle command handlers.
//!
//! Handles SQL mutations for asset creation (single and batch),
//! move-detection recovery, and physical/logical deletion via the Outbox pattern.
use crate::core::error::{AppError, AppResult};
use crate::core::ledger::command::CreateAssetPayload;
use crate::core::models::asset::Asset;
use crate::infra::database::ledger::SqliteAssetLedger;
use chrono::Utc;
use sqlx::{Sqlite, Transaction};

use uuid::Uuid;

/// Handles the creation of a new asset.
///
/// Executes an UPSERT operation for the asset and resolves move signatures.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `payload` - The payload containing the asset details.
///
/// # Errors
///
/// Returns `AppError` if validation fails or the database query encounters an error.
pub async fn handle_create(
    tx: &mut Transaction<'_, Sqlite>,
    payload: CreateAssetPayload,
) -> AppResult<Asset> {
    let asset_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let name = payload
        .path
        .file_name()
        .and_then(|file_name_os| file_name_os.to_str())
        .ok_or_else(|| AppError::ValidationFailed("Invalid file path".to_string()))?;
    let path_string = payload.path.to_string_lossy().to_string();
    let state_string = payload.state_init.to_string();
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

        for (existing_asset_id, _old_folder_id, old_path_string) in &move_candidates {
            if !std::path::Path::new(old_path_string).exists() && old_path_string != &path_string {
                tracing::info!(
                    "Ledger: MOVE DETECTED (V1 recovery). Updating asset {} from '{}' to '{}'",
                    existing_asset_id, old_path_string, path_string
                );

                let folder_id_for_update = payload.folder_id.as_deref();
                sqlx::query!(
                    "UPDATE assets SET path = ?, name = ?, folder_id = ?, modified_at = ?, updated_at = ?, state = ? WHERE id = ?",
                    path_string,
                    name,
                    folder_id_for_update,
                    modified_at_val,
                    now,
                    state_string,
                    existing_asset_id
                )
                .execute(&mut **tx)
                .await?;

                SqliteAssetLedger::log_operation(
                    tx,
                    "CREATE_ASSET_MOVE_RECOVERY",
                    existing_asset_id,
                    serde_json::json!({
                        "old_path": old_path_string,
                        "new_path": path_string,
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

    let path_reference = &path_string;
    let state_reference = &state_string;
    let format_type_reference = &payload.format_type;
    let family_reference = &payload.family;
    let folder_id_reference = payload.folder_id.as_deref();
    let asset_id_final_reference = &asset_id;

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
        asset_id_final_reference,
        name,
        path_reference,
        state_reference,
        format_type_reference,
        family_reference,
        file_size_i64,
        created_at_val,
        modified_at_val,
        added_at_val,
        now,
        folder_id_reference
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

/// Handles the batch creation of multiple assets.
///
/// This avoids excessive locks and emits a unified response.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `payloads` - A vector of create payloads.
///
/// # Errors
///
/// Returns `AppError` if validation fails or the database query encounters an error.
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
            .and_then(|file_name_os| file_name_os.to_str())
            .ok_or_else(|| {
                AppError::ValidationFailed("Invalid file path".to_string())
            })?;
        let state_string = payload.state_init.to_string();
        let path_string = payload.path.to_string_lossy().to_string();
        let file_size_i64 = payload.file_size as i64;

        let created_at_val = payload.created_at.unwrap_or(now);
        let modified_at_val = payload.modified_at.unwrap_or(now);
        let added_at_val = now;

        let path_reference = &path_string;
        let state_reference = &state_string;
        let format_type_reference = &payload.format_type;
        let family_reference = &payload.family;
        let folder_id_reference = payload.folder_id.as_deref();
        let asset_id_reference = &asset_id;

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
            asset_id_reference,
            name,
            path_reference,
            state_reference,
            format_type_reference,
            family_reference,
            file_size_i64,
            created_at_val,
            modified_at_val,
            added_at_val,
            now,
            folder_id_reference
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

/// Handles the deletion of an asset.
///
/// Supports logical deletion (tombstone) and physical deletion via the Saga/Outbox pattern.
///
/// # Arguments
///
/// * `tx` - The database transaction.
/// * `asset_id` - The optional ID of the asset.
/// * `path` - The optional file path of the asset.
/// * `physical_delete` - Whether to physically delete the file on disk.
///
/// # Errors
///
/// Returns `AppError` if the asset is not found or database constraints fail.
pub async fn handle_delete_asset(
    tx: &mut Transaction<'_, Sqlite>,
    asset_id: Option<String>,
    path: Option<std::path::PathBuf>,
    physical_delete: bool,
) -> AppResult<Asset> {
    let path_string = path
        .as_ref()
        .map(|path_reference| path_reference.to_string_lossy().to_string())
        .unwrap_or_else(|| "None".to_string());
    tracing::info!(
        "Ledger: DeleteAsset START. asset_id: {:?}, path: {}",
        asset_id, path_string
    );

    // 1. Resolve Asset ID (Using robust fallback for macOS Unicode consistency)
    let resolved_id: String = match (asset_id, path.clone()) {
        (Some(id), _) => {
            tracing::info!("Ledger: DeleteAsset resolved by ID: {}", id);
            id.clone()
        }
        (None, Some(path_reference)) => match SqliteAssetLedger::resolve_asset_id_robust(tx, &path_reference).await? {
            Some(id) => {
                tracing::info!(
                    "Ledger: DeleteAsset resolved path '{}' to ID: {}",
                    path_reference.display(),
                    id
                );
                id
            }
            None => {
                tracing::warn!("Ledger: DeleteAsset IGNORED - path '{}' not found in DB (even after robust fallback)", path_reference.display());
                return Err(AppError::NotFound(format!(
                    "Asset not found at path: {}",
                    path_reference.display()
                )));
            }
        },
        _ => {
            tracing::error!("Ledger: DeleteAsset FAILED - missing ID and path");
            return Err(AppError::ValidationFailed(
                "DeleteAsset requires either asset_id or path".to_string(),
            ));
        }
    };

    // 2. Capture pre-deletion metadata for domain event enrichment
    let pre_delete_row = sqlx::query!(
        r#"SELECT folder_id as "folder_id?", name as "name!" FROM assets WHERE id = ?"#,
        resolved_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    let pre_delete_folder_id = pre_delete_row.as_ref().and_then(|row| row.folder_id.clone());
    let pre_delete_name = pre_delete_row.as_ref().map(|row| row.name.clone()).unwrap_or_else(|| "deleted".to_string());

    // 3. Perform Delete
    tracing::info!(
        "Ledger: DeleteAsset executing DELETE for ID {}",
        resolved_id
    );
    sqlx::query!("DELETE FROM assets WHERE id = ?", resolved_id)
        .execute(&mut **tx)
        .await?;

    // 4. Audit Log - Use Outbox pattern (PENDING if physical, COMPLETED otherwise)
    let status = if physical_delete { "PENDING" } else { "COMPLETED" };
    SqliteAssetLedger::log_operation(
        tx,
        "DELETE_ASSET",
        &resolved_id,
        serde_json::json!({"physical": physical_delete, "path": path.map(|p| p.to_string_lossy().to_string())}),
        status,
        None,
    )
    .await?;

    tracing::info!("Ledger: DeleteAsset SUCCESS for ID {}", resolved_id);
    // 5. Return Tombstone (with original folder_id for event enrichment)
    Ok(Asset {
        id: resolved_id,
        name: pre_delete_name,
        path: std::path::PathBuf::new(),
        state: crate::core::models::asset::AssetState::Offline,
        format_type: "".to_string(),
        family: "".to_string(),
        file_size: 0,
        created_at: None,
        modified_at: None,
        added_at: None,
        updated_at: None,
        width: None,
        height: None,
        duration_secs: None,
        technical_payload: None,
        semantic_payload: None,
        dominant_color: None,
        folder_id: pre_delete_folder_id,
        thumbnail_path: None,
        rating: None,
        notes: None,
    })
}
