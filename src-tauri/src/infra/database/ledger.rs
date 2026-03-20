use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};
use crate::core::events::{AppEventBus, DomainEvent};
use crate::core::ledger::command::{
    LedgerCommand, UpdateAssetNotesPayload, UpdateAssetRatingPayload, UpdateTechnicalMetadataPayload,
};
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::{Asset, AssetState};

/// SQLite implementation of the Asset Ledger using SQLx.
///
/// This adapter ensures that all mutations are atomic and audited
/// via the `asset_operations_log` table before publishing domain events.
pub struct SqliteAssetLedger {
    pool: SqlitePool,
    event_bus: Arc<dyn AppEventBus>,
}

impl SqliteAssetLedger {
    /// Creates a new instance of the SqliteAssetLedger.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool.
    /// * `event_bus` - The event bus for publishing domain events.
    ///
    /// # Returns
    ///
    /// A new instance of `SqliteAssetLedger`.
    pub fn new(pool: SqlitePool, event_bus: Arc<dyn AppEventBus>) -> Self {
        Self { pool, event_bus }
    }

    /// Records an operation in the audit log.
    ///
    /// # Arguments
    ///
    /// * `tx` - The database transaction.
    /// * `operation_type` - The type of operation.
    /// * `asset_id` - The ID of the asset.
    /// * `payload` - The payload of the operation.
    /// * `status` - The status of the operation.
    /// * `error_note` - The error note of the operation.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn log_operation(
        tx: &mut Transaction<'_, Sqlite>,
        operation_type: &str,
        asset_id: &str,
        payload: serde_json::Value,
        status: &str,
        error_note: Option<&str>,
    ) -> AppResult<()> {
        let op_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO asset_operations_log (id, operation_type, asset_id, payload, status, error_note, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            op_id,
            operation_type,
            asset_id,
            payload,
            status,
            error_note,
            now
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn fetch_asset_by_id(
        tx: &mut Transaction<'_, Sqlite>,
        asset_id: &str,
    ) -> AppResult<Asset> {
        let row = sqlx::query!(
            r#"
            SELECT
                a.id as "id!", a.name as "name!", a.path as "path!", a.state as "state!",
                a.format_type as "format_type!", a.family as "family!", a.file_size as "file_size!",
                a.created_at as "created_at: DateTime<Utc>",
                a.modified_at as "modified_at: DateTime<Utc>",
                a.added_at as "added_at: DateTime<Utc>",
                a.updated_at as "updated_at: DateTime<Utc>",
                a.folder_id as "folder_id?",
                a.thumbnail_path as "thumbnail_path?",
                a.rating as "rating: i64",
                a.notes as "notes?",
                ame.width as "width: i64",
                ame.height as "height: i64",
                ame.duration_secs as "duration_secs: f64",
                ame.technical_payload as "technical_payload: serde_json::Value",
                ame.semantic_payload as "semantic_payload: serde_json::Value",
                a.dominant_color as "dominant_color: serde_json::Value"
            FROM assets a
            LEFT JOIN asset_metadata_envelope ame ON a.id = ame.asset_id
            WHERE a.id = ?
            "#,
            asset_id
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound(asset_id.to_string()))?;

        let asset_db = crate::infra::database::models::AssetDb {
            id: row.id,
            name: row.name,
            path: row.path,
            state: row.state,
            format_type: row.format_type,
            family: row.family,
            file_size: row.file_size,
            created_at: row.created_at,
            modified_at: row.modified_at,
            added_at: row.added_at,
            updated_at: row.updated_at,
            folder_id: row.folder_id,
            thumbnail_path: row.thumbnail_path,
            rating: row.rating,
            notes: row.notes,
            width: row.width,
            height: row.height,
            duration_secs: row.duration_secs,
            technical_payload: row.technical_payload,
            semantic_payload: row.semantic_payload,
            dominant_color: row.dominant_color,
        };

        Ok(asset_db.into())
    }

    async fn handle_update_rating(
        &self,
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

        Self::log_operation(
            tx,
            "UPDATE_ASSET_RATING",
            &payload.asset_id,
            serde_json::json!({ "rating": payload.rating }),
            "COMPLETED",
            None,
        )
        .await?;

                Self::fetch_asset_by_id(tx, &payload.asset_id).await
    }

    async fn handle_update_notes(
        &self,
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

        Self::log_operation(
            tx,
            "UPDATE_ASSET_NOTES",
            &payload.asset_id,
            serde_json::json!({ "notes": payload.notes }),
            "COMPLETED",
            None,
        )
        .await?;

                Self::fetch_asset_by_id(tx, &payload.asset_id).await
    }

    async fn handle_reextract_colors(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        asset_id: &str,
    ) -> AppResult<Asset> {
        // 1. Get thumbnail path
        let asset_row = sqlx::query!(
            r#"SELECT thumbnail_path as "thumbnail_path?", format_type as "format_type!" FROM assets WHERE id = ?"#,
            asset_id
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound(asset_id.to_string()))?;

        if let Some(path) = asset_row.thumbnail_path {
            // 2. Clear existing colors
            sqlx::query!("DELETE FROM asset_colors WHERE asset_id = ?", asset_id)
                .execute(&mut **tx)
                .await?;

            // 3. Emit event (this will be handled by ColorWorker)
            self.event_bus.publish(DomainEvent::ThumbnailGenerated {
                asset_id: asset_id.to_string(),
                path,
                format: asset_row.format_type,
            })?;
        }

        Self::log_operation(
            tx,
            "REEXTRACT_COLORS",
            asset_id,
            serde_json::json!({}),
            "COMPLETED",
            None,
        )
        .await?;

        Self::fetch_asset_by_id(tx, &asset_id).await
    }

    async fn handle_update_technical_metadata(
        &self,
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

        Self::log_operation(
            tx,
            "UPDATE_TECHNICAL_METADATA",
            &payload.asset_id,
            serde_json::to_value(&payload).unwrap_or_default(),
            "COMPLETED",
            None,
        )
        .await?;

        Self::fetch_asset_by_id(tx, &payload.asset_id).await
    }
}

/// Implementation of the TransactionalAssetLedger trait for SqliteAssetLedger.
///
/// This trait is used to execute commands that modify the state of the asset ledger.
///
/// # Arguments
///
/// * `command` - The command to execute.
///
/// # Returns
///
/// A `Result` containing the updated asset or an error.
#[async_trait]
impl TransactionalAssetLedger for SqliteAssetLedger {
    /// Executes a command that modifies the state of the asset ledger.
    async fn execute(&self, command: LedgerCommand) -> AppResult<Asset> {
        let mut tx = self.pool.begin().await?;

        // 1. Resolve and expand commands (Handle Batch and BatchCreate expansion)
        let commands_to_process = match &command {
            LedgerCommand::Batch(cmds) => cmds.clone(),
            LedgerCommand::BatchCreate(payloads) => payloads
                .iter()
                .map(|p| LedgerCommand::CreateAsset(p.clone()))
                .collect(),
            _ => vec![command.clone()],
        };

        // 2. Execute commands and collect results
        let mut results = Vec::new();
        for cmd in commands_to_process {
            let asset = self.execute_single(&mut tx, cmd.clone()).await?;
            results.push((asset, cmd));
        }

        tx.commit().await?;

        // 2. Publish Domain Events only AFTER commit
        // Use the specific asset associated with each command
        for (asset, cmd) in &results {
            self.emit_event_for_command(asset, cmd)?;
        }

        // Return the last asset in the batch (or the only asset if not a batch)
        let last_asset = results.into_iter().last().map(|(a, _)| a).ok_or_else(|| {
            AppError::Internal("Transaction succeeded but no results were returned".to_string())
        })?;

        Ok(last_asset)
    }
}

impl SqliteAssetLedger {
    /// Internal helper to emit domain events after a successful commit.
    fn emit_event_for_command(&self, asset: &Asset, command: &LedgerCommand) -> AppResult<()> {
        match command {
            LedgerCommand::CreateAsset(_) | LedgerCommand::BatchCreate(_) => {
                self.event_bus.publish(DomainEvent::AssetCreated {
                    asset_id: asset.id.clone(),
                    path: asset.path.to_string_lossy().to_string(),
                    format: asset.format_type.clone(),
                })?;
            }
            LedgerCommand::UpdateTags(p) => {
                self.event_bus.publish(DomainEvent::AssetTagsUpdated {
                    asset_id: asset.id.clone(),
                    active_tags: p.tags_to_add.clone(),
                })?;
            }
            LedgerCommand::UpdateAsset(_) => {
                self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                    asset_id: asset.id.clone(),
                })?;
            }
            LedgerCommand::DeleteAsset { .. } => {
                self.event_bus.publish(DomainEvent::FsPathDeleted {
                    path: asset.id.clone(),
                })?;
            }
            LedgerCommand::CreateFolder(_) => {
                self.event_bus.publish(DomainEvent::FolderCreated {
                    folder_id: asset.id.clone(),
                    parent_id: asset.folder_id.clone(),
                    name: asset.name.clone(),
                    path: asset.path.to_string_lossy().to_string(),
                })?;
            }
            LedgerCommand::SetAssetFolder { .. } => {
                self.event_bus.publish(DomainEvent::AssetFolderChanged {
                    asset_id: asset.id.clone(),
                    folder_id: asset.folder_id.clone(),
                })?;
            }
            LedgerCommand::UpdateThumbnail { thumbnail_path, .. } => {
                self.event_bus.publish(DomainEvent::ThumbnailGenerated {
                    asset_id: asset.id.clone(),
                    path: thumbnail_path.clone(),
                    format: asset.format_type.clone(),
                })?;
            }
            LedgerCommand::UpdateAssetColors(_) => {
                self.event_bus.publish(DomainEvent::ExtractionCompleted {
                    asset_id: asset.id.clone(),
                    capability: "COLORS".to_string(),
                })?;
            }
            LedgerCommand::CreateTag(_) => {
                self.event_bus.publish(DomainEvent::TagCreated {
                    id: asset.id.clone(),
                    name: asset.name.clone(),
                })?;
            }
            LedgerCommand::UpdateTag(tag_payload) => {
                self.event_bus.publish(DomainEvent::TagUpdated {
                    id: tag_payload.id.clone(),
                })?;
            }
            LedgerCommand::DeleteTag { id } => {
                self.event_bus
                    .publish(DomainEvent::TagDeleted { id: id.clone() })?;
            }
            LedgerCommand::CreateSmartFolder(_) => {
                self.event_bus.publish(DomainEvent::SmartFolderCreated {
                    id: asset.id.clone(),
                    name: asset.name.clone(),
                })?;
            }
            LedgerCommand::UpdateSmartFolder(sf_payload) => {
                self.event_bus.publish(DomainEvent::SmartFolderUpdated {
                    id: sf_payload.id.clone(),
                })?;
            }
            LedgerCommand::DeleteSmartFolder(sf_payload) => {
                self.event_bus.publish(DomainEvent::SmartFolderDeleted {
                    id: sf_payload.id.clone(),
                })?;
            }
            LedgerCommand::UpdateAssetRating(p) => {
                self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                    asset_id: p.asset_id.clone(),
                })?;
            }
            LedgerCommand::UpdateAssetNotes(p) => {
                self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                    asset_id: p.asset_id.clone(),
                })?;
            }
            LedgerCommand::UpdateTechnicalMetadata(p) => {
                self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                    asset_id: p.asset_id.clone(),
                })?;
            }
            LedgerCommand::ReextractColors { asset_id } => {
                self.event_bus.publish(DomainEvent::ReextractAssetColors {
                    asset_id: asset_id.clone(),
                })?;
            }
            LedgerCommand::RegenerateThumbnail { asset_id } => {
                self.event_bus.publish(DomainEvent::ThumbnailInvalidated {
                    asset_id: asset_id.clone(),
                })?;
            }
            LedgerCommand::AddTagsToAssetsBatch(p)
            | LedgerCommand::RemoveTagsFromAssetsBatch(p)
            | LedgerCommand::ReplaceTagsForAssetsBatch(p) => {
                for asset_id in &p.asset_ids {
                    self.event_bus.publish(DomainEvent::AssetTagsUpdated {
                        asset_id: asset_id.clone(),
                        active_tags: p.tag_ids.clone(),
                    })?;
                }
            }
            _ => {
                // Other commands without specific domain events
            }
        }
        Ok(())
    }

    /// Optimized single command execution within an existing transaction.
    async fn execute_single(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        command: LedgerCommand,
    ) -> AppResult<Asset> {
        match command {
            LedgerCommand::CreateAsset(payload) => {
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

                let path_ref = &path_str;
                let state_ref = &state_str;
                let format_type_ref = &payload.format_type;
                let family_ref = &payload.family;
                let folder_id_ref = payload.folder_id.as_deref();
                let asset_id_final_ref = &asset_id;

                let created_at_val = payload.created_at.unwrap_or(now);
                let modified_at_val = payload.modified_at.unwrap_or(now);
                let added_at_val = now;

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

                Self::log_operation(
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
            LedgerCommand::BatchCreate(payloads) => {
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

                    let path_ref = &path_str;
                    let state_ref = &state_str;
                    let format_type_ref = &payload.format_type;
                    let family_ref = &payload.family;
                    let folder_id_ref = payload.folder_id.as_deref();
                    let asset_id_ref = &asset_id;

                    let created_at_val = payload.created_at.unwrap_or(now);
                    let modified_at_val = payload.modified_at.unwrap_or(now);
                    let added_at_val = now;

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

                    Self::log_operation(
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

                // For batch, we just return the "last" or a dummy if needed,
                // but the trait says result must be AppResult<Asset>.
                // We'll return the first one or a dummy.
                created_assets
                    .into_iter()
                    .next()
                    .ok_or_else(|| AppError::ValidationFailed("Empty batch".to_string()))
            }
            LedgerCommand::UpdateTags(payload) => {
                let now = Utc::now();

                // 1. Add Tags by ID
                for tag_id in &payload.tags_to_add {
                    sqlx::query!(
                        r#"
                        INSERT INTO asset_tags (asset_id, tag_id)
                        VALUES (?, ?)
                        ON CONFLICT DO NOTHING
                        "#,
                        payload.asset_id,
                        tag_id
                    )
                    .execute(&mut **tx)
                    .await?;
                }

                // 2. Remove Tags by ID
                for tag_id in &payload.tags_to_remove {
                    sqlx::query!(
                        r#"
                        DELETE FROM asset_tags
                        WHERE asset_id = ? AND tag_id = ?
                        "#,
                        payload.asset_id,
                        tag_id
                    )
                    .execute(&mut **tx)
                    .await?;
                }

                // 3. Update Asset timestamp
                sqlx::query!(
                    "UPDATE assets SET updated_at = ? WHERE id = ?",
                    now,
                    payload.asset_id
                )
                .execute(&mut **tx)
                .await?;
                // 4. Audit Log
                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "UPDATE_TAGS",
                    &payload.asset_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Self::fetch_asset_by_id(tx, &payload.asset_id).await
            }
            LedgerCommand::UpdateAsset(payload) => {
                let now = Utc::now();

                // 1. Resolve Asset ID
                let asset_id: String = match (&payload.asset_id, &payload.old_path) {
                    (Some(id), _) => id.clone(),
                    (None, Some(old_path)) => {
                        let old_path_str = old_path.to_string_lossy().to_string();
                        let row = sqlx::query!(
                            r#"SELECT id as "id!" FROM assets WHERE path = ?"#,
                            old_path_str
                        )
                        .fetch_optional(&mut **tx)
                        .await?;
                        row.map(|r| r.id.to_string()).ok_or_else(|| {
                            AppError::NotFound(format!("Asset not found at path: {}", old_path_str))
                        })?
                    }
                    _ => {
                        return Err(AppError::ValidationFailed(
                            "UpdateAsset requires either asset_id or old_path".to_string(),
                        ))
                    }
                };

                let new_name = payload
                    .new_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| {
                        AppError::ValidationFailed("Invalid new file path".to_string())
                    })?;
                let new_path_str = payload.new_path.to_string_lossy().to_string();

                // 2. Update Asset
                sqlx::query!(
                    "UPDATE assets SET path = ?, name = ?, updated_at = ? WHERE id = ?",
                    new_path_str,
                    new_name,
                    now,
                    asset_id
                )
                .execute(&mut **tx)
                .await?;

                // 3. Audit Log
                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "UPDATE_ASSET",
                    &asset_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // 4. Fetch and return
        Self::fetch_asset_by_id(tx, &asset_id).await
            }
            LedgerCommand::MarkAsStale { asset_id } => {
                let now = Utc::now();
                let state_stale = AssetState::Stale.to_string();

                sqlx::query!(
                    "UPDATE assets SET state = ?, updated_at = ? WHERE id = ?",
                    state_stale,
                    now,
                    asset_id
                )
                .execute(&mut **tx)
                .await?;

                Self::log_operation(
                    tx,
                    "MARK_STALE",
                    &asset_id,
                    serde_json::json!({}),
                    "COMPLETED",
                    None,
                )
                .await?;

                Self::fetch_asset_by_id(tx, &asset_id).await
            }
            LedgerCommand::DeleteAsset {
                asset_id,
                path,
                physical_delete,
            } => {
                // 1. Resolve Asset ID
                let resolved_id: String = match (asset_id, path) {
                    (Some(id), _) => id.clone(),
                    (None, Some(p)) => {
                        let path_str = p.to_string_lossy().to_string();
                        let row = sqlx::query!(
                            r#"SELECT id as "id!" FROM assets WHERE path = ?"#,
                            path_str
                        )
                        .fetch_optional(&mut **tx)
                        .await?;
                        row.map(|r| r.id.to_string()).ok_or_else(|| {
                            AppError::NotFound(format!("Asset not found at path: {}", path_str))
                        })?
                    }
                    _ => {
                        return Err(AppError::ValidationFailed(
                            "DeleteAsset requires either asset_id or path".to_string(),
                        ))
                    }
                };

                // 2. Perform Delete
                sqlx::query!("DELETE FROM assets WHERE id = ?", resolved_id)
                    .execute(&mut **tx)
                    .await?;

                // 3. Audit Log
                Self::log_operation(
                    tx,
                    "DELETE_ASSET",
                    &resolved_id,
                    serde_json::json!({"physical": physical_delete}),
                    "COMPLETED",
                    None,
                )
                .await?;

                // 4. Return Tombstone
                Ok(Asset {
                    id: resolved_id,
                    name: "deleted".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Offline,
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::CreateFolder(payload) => {
                let folder_id = Uuid::new_v4().to_string();
                let now = Utc::now();
                let path_str = payload.path.to_string_lossy().to_string();

                sqlx::query!(
                    r#"
                    INSERT INTO folders (id, parent_id, name, path, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    "#,
                    folder_id,
                    payload.parent_id,
                    payload.name,
                    path_str,
                    now,
                    now
                )
                .execute(&mut **tx)
                .await?;

                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "CREATE_FOLDER",
                    &folder_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // Return a dummy asset or update trait to handle Folder return
                // Since TransactionalAssetLedger::execute returns AppResult<Asset>,
                // we return a "Virtual Asset" representing the folder or just a tombstone.
                // Re-evaluating: In a proper CQRS, CreateFolder might return a different type,
                // but let's stick to the trait and return a dummy for now.
                Ok(Asset {
                    id: folder_id,
                    name: payload.name.clone(),
                    path: payload.path.clone(),
                    state: AssetState::Idle,
                    format_type: "folder".to_string(),
                    family: "FOLDER".to_string(),
                    file_size: 0,
                    created_at: Some(now),
                    modified_at: Some(now),
                    added_at: Some(now),
                    updated_at: Some(now),
                    width: None,
                    height: None,
                    duration_secs: None,
                    technical_payload: None,
                    semantic_payload: None,
                    dominant_color: None,
                    folder_id: payload.parent_id.clone(),
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::RemoveFolder(payload) => {
                let folder_id_ref = &payload.folder_id;

                // 1. Get folder path to include in the event
                let folder_path = sqlx::query!(
                    r#"SELECT path as "path!" FROM folders WHERE id = ?"#,
                    folder_id_ref
                )
                .fetch_optional(&mut **tx)
                .await?
                .map(|r| r.path)
                .ok_or_else(|| {
                    AppError::NotFound(format!("Folder ID not found: {}", folder_id_ref))
                })?;

                // 2. Perform Cascade Delete
                // To be safe with tags and colors, we use the recursive CTE to find all subfolders
                let all_folder_ids = sqlx::query!(
                    r#"
                    WITH RECURSIVE family AS (
                        SELECT id FROM folders WHERE id = ?
                        UNION ALL
                        SELECT f.id FROM folders f JOIN family ON f.parent_id = family.id
                    )
                    SELECT id as "id!" FROM family
                    "#,
                    folder_id_ref
                )
                .fetch_all(&mut **tx)
                .await?;

                for record in all_folder_ids {
                    // Manual cascade: delete assets for this specific subfolder first
                    sqlx::query!("DELETE FROM assets WHERE folder_id = ?", record.id)
                        .execute(&mut **tx)
                        .await?;

                    sqlx::query!("DELETE FROM folders WHERE id = ?", record.id)
                        .execute(&mut **tx)
                        .await?;
                }

                // 3. Audit Log
                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "REMOVE_FOLDER",
                    folder_id_ref,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // Return a dummy/tombstone Asset
                Ok(Asset {
                    id: payload.folder_id.clone(),
                    name: "deleted_folder".to_string(),
                    path: std::path::PathBuf::from(folder_path),
                    state: AssetState::Offline,
                    format_type: "folder".to_string(),
                    family: "FOLDER".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::SetAssetFolder {
                asset_id,
                folder_id,
            } => {
                let now = Utc::now();

                sqlx::query!(
                    "UPDATE assets SET folder_id = ?, updated_at = ? WHERE id = ?",
                    folder_id,
                    now,
                    asset_id
                )
                .execute(&mut **tx)
                .await?;

                Self::log_operation(
                    tx,
                    "SET_ASSET_FOLDER",
                    &asset_id,
                    serde_json::json!({ "folder_id": folder_id }),
                    "COMPLETED",
                    None,
                )
                .await?;

                Self::fetch_asset_by_id(tx, &asset_id).await
            }
            LedgerCommand::UpdateThumbnail {
                asset_id,
                thumbnail_path,
            } => {
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
                Self::log_operation(
                    tx,
                    "UPDATE_THUMBNAIL",
                    &asset_id,
                    serde_json::json!({ "path": thumbnail_path }),
                    "COMPLETED",
                    None,
                )
                .await?;

                // 3. Fetch and return
        Self::fetch_asset_by_id(tx, &asset_id).await
            }
            LedgerCommand::UpdateAssetColors(payload) => {
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

                Self::log_operation(
                    tx,
                    "UPDATE_ASSET_COLORS",
                    asset_id_ref,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Self::fetch_asset_by_id(tx, &payload.asset_id).await
            }
            LedgerCommand::UpdateAssetRating(payload) => {
                self.handle_update_rating(tx, payload.clone()).await
            }
            LedgerCommand::UpdateAssetNotes(payload) => {
                self.handle_update_notes(tx, payload.clone()).await
            }
            LedgerCommand::ReextractColors { asset_id } => {
                self.handle_reextract_colors(tx, &asset_id).await
            }
            LedgerCommand::UpdateTechnicalMetadata(payload) => {
                self.handle_update_technical_metadata(tx, payload.clone()).await
            }

            // ── Tag CRUD Handlers ──────────────────────────────────────────
            LedgerCommand::CreateTag(payload) => {
                let tag_id = Uuid::new_v4().to_string();

                let normalized_parent_id = payload.parent_id.as_ref().and_then(|id| {
                    if id.is_empty() || id == "0" {
                        None
                    } else {
                        Some(id.clone())
                    }
                });

                sqlx::query!(
                    r#"INSERT INTO tags (id, name, color, parent_id, order_index) VALUES (?, ?, ?, ?, ?)"#,
                    tag_id,
                    payload.name,
                    payload.color,
                    normalized_parent_id,
                    0 // Default order_index for new tags
                )
                .execute(&mut **tx)
                .await?;

                let operation_payload =
                    serde_json::to_value(&payload).map_err(|serialization_error| {
                        AppError::Internal(format!(
                            "Failed to serialize payload: {}",
                            serialization_error
                        ))
                    })?;

                Self::log_operation(
                    tx,
                    "CREATE_TAG",
                    &tag_id,
                    operation_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // Return dummy Asset carrying the tag_id in the id field
                Ok(Asset {
                    id: tag_id,
                    name: payload.name.clone(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "tag".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::UpdateTag(payload) => {
                // Build a dynamic UPDATE query only for non-None fields
                let mut set_clauses = Vec::new();

                if payload.name.is_some() {
                    set_clauses.push("name = ?");
                }
                if payload.color.is_some() {
                    set_clauses.push("color = ?");
                }
                if payload.parent_id.is_some() {
                    set_clauses.push("parent_id = ?");
                }
                if payload.order_index.is_some() {
                    set_clauses.push("order_index = ?");
                }

                if !set_clauses.is_empty() {
                    let update_sql =
                        format!("UPDATE tags SET {} WHERE id = ?", set_clauses.join(", "));
                    let mut query = sqlx::query(&update_sql);

                    if let Some(ref tag_name) = payload.name {
                        query = query.bind(tag_name);
                    }
                    if let Some(ref tag_color) = payload.color {
                        query = query.bind(tag_color);
                    }
                    if let Some(ref parent_tag_id) = payload.parent_id {
                        if parent_tag_id.is_empty() || parent_tag_id == "0" {
                            query = query.bind(None::<String>);
                        } else {
                            query = query.bind(parent_tag_id);
                        }
                    }
                    if let Some(sort_order) = payload.order_index {
                        query = query.bind(sort_order);
                    }

                    query = query.bind(&payload.id);
                    query.execute(&mut **tx).await?;
                }

                let operation_payload =
                    serde_json::to_value(&payload).map_err(|serialization_error| {
                        AppError::Internal(format!(
                            "Failed to serialize payload: {}",
                            serialization_error
                        ))
                    })?;

                Self::log_operation(
                    tx,
                    "UPDATE_TAG",
                    &payload.id,
                    operation_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: payload.id.clone(),
                    name: "updated_tag".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "tag".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::DeleteTag { id } => {
                // 1. Remove all asset associations first
                sqlx::query!("DELETE FROM asset_tags WHERE tag_id = ?", id)
                    .execute(&mut **tx)
                    .await?;

                // 2. Delete the tag itself
                sqlx::query!("DELETE FROM tags WHERE id = ?", id)
                    .execute(&mut **tx)
                    .await?;

                Self::log_operation(
                    tx,
                    "DELETE_TAG",
                    &id,
                    serde_json::json!({ "tag_id": id }),
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: id.clone(),
                    name: "deleted_tag".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Offline,
                    format_type: "tag".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::AddTagsToAssetsBatch(payload) => {
                if !payload.asset_ids.is_empty() && !payload.tag_ids.is_empty() {
                    for current_asset_id in &payload.asset_ids {
                        for current_tag_id in &payload.tag_ids {
                            sqlx::query!(
                                "INSERT INTO asset_tags (asset_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
                                current_asset_id,
                                current_tag_id
                            )
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                let operation_payload =
                    serde_json::to_value(&payload).map_err(|serialization_error| {
                        AppError::Internal(format!(
                            "Failed to serialize payload: {}",
                            serialization_error
                        ))
                    })?;

                Self::log_operation(
                    tx,
                    "ADD_TAGS_BATCH",
                    "batch",
                    operation_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: "batch_add_tags".to_string(),
                    name: "batch_operation".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "batch".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::RemoveTagsFromAssetsBatch(payload) => {
                if !payload.asset_ids.is_empty() && !payload.tag_ids.is_empty() {
                    for current_asset_id in &payload.asset_ids {
                        for current_tag_id in &payload.tag_ids {
                            sqlx::query!(
                                "DELETE FROM asset_tags WHERE asset_id = ? AND tag_id = ?",
                                current_asset_id,
                                current_tag_id
                            )
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                let operation_payload =
                    serde_json::to_value(&payload).map_err(|serialization_error| {
                        AppError::Internal(format!(
                            "Failed to serialize payload: {}",
                            serialization_error
                        ))
                    })?;

                Self::log_operation(
                    tx,
                    "REMOVE_TAGS_BATCH",
                    "batch",
                    operation_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: "batch_remove_tags".to_string(),
                    name: "batch_operation".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "batch".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::ReplaceTagsForAssetsBatch(payload) => {
                if !payload.asset_ids.is_empty() {
                    for current_asset_id in &payload.asset_ids {
                        // Remove all existing tags for this asset
                        sqlx::query!(
                            "DELETE FROM asset_tags WHERE asset_id = ?",
                            current_asset_id
                        )
                        .execute(&mut **tx)
                        .await?;

                        // Add the new set of tags
                        for current_tag_id in &payload.tag_ids {
                            sqlx::query!(
                                "INSERT INTO asset_tags (asset_id, tag_id) VALUES (?, ?) ON CONFLICT DO NOTHING",
                                current_asset_id,
                                current_tag_id
                            )
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                let operation_payload =
                    serde_json::to_value(&payload).map_err(|serialization_error| {
                        AppError::Internal(format!(
                            "Failed to serialize payload: {}",
                            serialization_error
                        ))
                    })?;

                Self::log_operation(
                    tx,
                    "REPLACE_TAGS_BATCH",
                    "batch",
                    operation_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: "batch_replace_tags".to_string(),
                    name: "batch_operation".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "batch".to_string(),
                    family: "TAG".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::CreateSmartFolder(payload) => {
                let sf_id = Uuid::new_v4().to_string();
                let now = Utc::now();

                sqlx::query!(
                    r#"INSERT INTO smart_folders (id, name, query_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"#,
                    sf_id,
                    payload.name,
                    payload.query_json,
                    now,
                    now
                )
                .execute(&mut **tx)
                .await?;

                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "CREATE_SMART_FOLDER",
                    &sf_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: sf_id,
                    name: payload.name.clone(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "smart_folder".to_string(),
                    family: "SMART_FOLDER".to_string(),
                    file_size: 0,
                    created_at: Some(now),
                                        modified_at: Some(now),
                    added_at: Some(now),
                    updated_at: Some(now),
                    width: None,
                    height: None,
                    duration_secs: None,
                    technical_payload: None,
                    semantic_payload: None,
                    dominant_color: None,
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::UpdateSmartFolder(payload) => {
                let now = Utc::now();

                sqlx::query!(
                    "UPDATE smart_folders SET name = ?, query_json = ?, updated_at = ? WHERE id = ?",
                    payload.name,
                    payload.query_json,
                    now,
                    payload.id
                )
                .execute(&mut **tx)
                .await?;

                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "UPDATE_SMART_FOLDER",
                    &payload.id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: payload.id.clone(),
                    name: "updated_smart_folder".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Idle,
                    format_type: "smart_folder".to_string(),
                    family: "SMART_FOLDER".to_string(),
                    file_size: 0,
                    created_at: None,
                                        modified_at: None,
                    added_at: None,
                    updated_at: Some(now),
                    width: None,
                    height: None,
                    duration_secs: None,
                    technical_payload: None,
                    semantic_payload: None,
                    dominant_color: None,
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::DeleteSmartFolder(payload) => {
                sqlx::query!("DELETE FROM smart_folders WHERE id = ?", payload.id)
                    .execute(&mut **tx)
                    .await?;

                let op_payload = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    tx,
                    "DELETE_SMART_FOLDER",
                    &payload.id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                Ok(Asset {
                    id: payload.id.clone(),
                    name: "deleted_smart_folder".to_string(),
                    path: std::path::PathBuf::new(),
                    state: AssetState::Offline,
                    format_type: "smart_folder".to_string(),
                    family: "SMART_FOLDER".to_string(),
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
                    folder_id: None,
                    thumbnail_path: None,
                    rating: None,
                    notes: None,
                })
            }
            LedgerCommand::RegenerateThumbnail { asset_id } => {
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
                Self::log_operation(
                    tx,
                    "REGENERATE_THUMBNAIL",
                    &asset_id,
                    serde_json::json!({ "asset_id": asset_id }),
                    "COMPLETED",
                    None,
                )
                .await?;

                // 3. Fetch asset to return
        Self::fetch_asset_by_id(tx, &asset_id).await
            }
            LedgerCommand::Batch(_) => {
                return Err(AppError::Internal("Nested Batch commands are not supported".to_string()));
            }
        }
    }
}
