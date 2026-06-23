//! SQLite implementation of the `TransactionalAssetLedger` port.
//!
//! Acts as the transactional router and domain-event emitter for all write
//! operations. It begins a SQLite transaction, delegates execution to the
//! appropriate specialized handler in `handlers/`, commits the transaction,
//! runs any post-commit Saga (Outbox) steps, and finally publishes domain
//! events on the `AppEventBus`.
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use tracing::{info, warn};
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};
use crate::core::events::{AppEventBus, DomainEvent};
use crate::core::ledger::command::LedgerCommand;
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
    pub(crate) async fn log_operation(
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

    pub(crate) async fn fetch_asset_by_id(
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

        Self::fetch_asset_by_id(tx, asset_id).await
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

        // 1. Resolve and expand commands (Handle Batch expansion but NOT BatchCreate)
        let commands_to_process = match &command {
            LedgerCommand::Batch(cmds) => cmds.clone(),
            // DO NOT MAP BatchCreate into CreateAsset!
            // It must be passed to execute_single as a single BatchCreate
            // so we don't trigger per-file lock-heavy checks.
            _ => vec![command.clone()],
        };

        // 2. Execute commands and collect results
        let mut results = Vec::new();
        for command_item in commands_to_process {
            let asset = self.execute_single(&mut tx, command_item.clone()).await?;
            results.push((asset, command_item));
        }

        tx.commit().await?;

        // 2.5 Post-commit Saga Execution (Filesystem Operations)
        for (asset, command_item) in &results {
            match command_item {
                LedgerCommand::DeleteAsset {
                    physical_delete: true,
                    path: Some(path_reference),
                    ..
                } => {
                    // Execute physical deletion
                    let filesystem_result = tokio::fs::remove_file(path_reference).await;
                    match filesystem_result {
                        Ok(_) => {
                            tracing::info!("Ledger: Physical delete SUCCESS for {}", path_reference.display());
                            // Mark Saga as COMPLETED
                            let _ = sqlx::query!(
                                "UPDATE asset_operations_log SET status = 'COMPLETED' WHERE asset_id = ? AND status = 'PENDING' AND operation_type = 'DELETE_ASSET'",
                                asset.id
                            )
                            .execute(&self.pool)
                            .await;
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            tracing::info!("Ledger: Physical file already missing for {}", path_reference.display());
                            let _ = sqlx::query!(
                                "UPDATE asset_operations_log SET status = 'COMPLETED' WHERE asset_id = ? AND status = 'PENDING' AND operation_type = 'DELETE_ASSET'",
                                asset.id
                            )
                            .execute(&self.pool)
                            .await;
                        }
                        Err(error) => {
                            tracing::warn!("Ledger: Physical delete FAILED for {}: {}", path_reference.display(), error);
                            let error_message = error.to_string();
                            let _ = sqlx::query!(
                                "UPDATE asset_operations_log SET status = 'FAILED', error_note = ? WHERE asset_id = ? AND status = 'PENDING' AND operation_type = 'DELETE_ASSET'",
                                error_message,
                                asset.id
                            )
                            .execute(&self.pool)
                            .await;
                        }
                    }
                }
                _ => {}
            }
        }

        // 3. Publish Domain Events only AFTER commit
        // For BatchCreate: emit a single summary to avoid flooding the broadcast channel.
        // Individual AssetCreated events would overflow the 2048-capacity buffer.
        match &command {
            LedgerCommand::BatchCreate(payloads) => {
                let batch_count = payloads.len();
                if batch_count > 0 {
                    let _ = self.event_bus.publish(DomainEvent::ScanProgress {
                        total: batch_count,
                        processed: batch_count,
                        current_file: format!("Batch committed: {} assets", batch_count),
                    });
                }
            }
            _ => {
                for (asset, command_item) in &results {
                    self.emit_event_for_command(asset, command_item)?;
                }
            }
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
                self.event_bus.publish(DomainEvent::AssetDeleted {
                    asset_id: asset.id.clone(),
                    folder_id: asset.folder_id.clone(),
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
            LedgerCommand::RenameFolder(payload) => {
                self.event_bus.publish(DomainEvent::FolderMetadataUpdated {
                    folder_id: payload.folder_id.clone(),
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
            LedgerCommand::UpdateFormat { asset_id, .. } => {
                self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                    asset_id: asset_id.clone(),
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
                crate::infra::database::handlers::asset_handler::handle_create(tx, payload).await
            }
            LedgerCommand::BatchCreate(payloads) => {
                crate::infra::database::handlers::asset_handler::handle_batch_create(tx, payloads).await
            }
            LedgerCommand::UpdateTags(payload) => {
                crate::infra::database::handlers::tags_handler::handle_update_tags(tx, payload).await
            }
            LedgerCommand::UpdateAsset(payload) => {
                let now = Utc::now();
                let old_path_string = payload
                    .old_path
                    .as_ref()
                    .map(|path_reference| path_reference.to_string_lossy().to_string())
                    .unwrap_or_else(|| "None".to_string());
                let new_path_string = payload.new_path.to_string_lossy().to_string();

                tracing::info!("Ledger: UpdateAsset START. old: {}, new: {}", old_path_string, new_path_string);

                // 1. Resolve Asset ID (Using robust fallback for macOS Unicode consistency)
                let asset_id: String = match (&payload.asset_id, &payload.old_path) {
                    (Some(id), _) => {
                        tracing::info!("Ledger: UpdateAsset resolved by ID: {}", id);
                        id.clone()
                    }
                    (None, Some(old_path)) => {
                        match Self::resolve_asset_id_robust(tx, old_path).await? {
                            Some(id) => {
                                tracing::info!(
                                    "Ledger: UpdateAsset resolved old_path '{}' to ID: {}",
                                    old_path.display(),
                                    id
                                );
                                id
                            }
                            None => {
                                tracing::warn!("Ledger: UpdateAsset IGNORED - old_path '{}' not found in DB (even after robust fallback)", old_path.display());
                                return Err(AppError::NotFound(format!(
                                    "Asset not found at path: {}",
                                    old_path.display()
                                )));
                            }
                        }
                    }
                    _ => {
                        tracing::error!("Ledger: UpdateAsset FAILED - missing both ID and old_path");
                        return Err(AppError::ValidationFailed(
                            "UpdateAsset requires either asset_id or old_path".to_string(),
                        ));
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

                // 2. Safety DELETE (Avoid Unique Constraint Violation on Rename)
                info!(
                    "Ledger: UpdateAsset safety DELETE checking for '{}' (collision prevention)",
                    new_path_str
                );
                let delete_res = sqlx::query!(
                    "DELETE FROM assets WHERE path = ? AND id != ?",
                    new_path_str,
                    asset_id
                )
                .execute(&mut **tx)
                .await?;

                if delete_res.rows_affected() > 0 {
                    info!(
                        "Ledger: UpdateAsset collision DETECTED. Pruned {} record(s) for '{}'",
                        delete_res.rows_affected(),
                        new_path_str
                    );
                }

                // 3. Update Asset
                info!(
                    "Ledger: UpdateAsset executing UPDATE for ID {} to NEW path '{}'",
                    asset_id, new_path_str
                );
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

                Self::log_operation(tx, "UPDATE_ASSET", &asset_id, op_payload, "COMPLETED", None)
                    .await?;

                info!("Ledger: UpdateAsset SUCCESS for ID {}", asset_id);
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
                crate::infra::database::handlers::asset_handler::handle_delete_asset(
                    tx,
                    asset_id,
                    path,
                    physical_delete,
                ).await
            }
            LedgerCommand::CreateFolder(payload) => {
                crate::infra::database::handlers::folder_handler::handle_create_folder(tx, payload).await
            }
            LedgerCommand::RemoveFolder(payload) => {
                crate::infra::database::handlers::folder_handler::handle_remove_folder(tx, payload).await
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
                crate::infra::database::handlers::thumbnail_handler::handle_update_thumbnail(tx, &asset_id, &thumbnail_path).await
            }
            LedgerCommand::UpdateAssetColors(payload) => {
                crate::infra::database::handlers::metadata_handler::handle_update_asset_colors(tx, payload).await
            }
            LedgerCommand::UpdateAssetRating(payload) => {
                crate::infra::database::handlers::metadata_handler::handle_update_rating(tx, payload).await
            }
            LedgerCommand::UpdateAssetNotes(payload) => {
                crate::infra::database::handlers::metadata_handler::handle_update_notes(tx, payload).await
            }
            LedgerCommand::ReextractColors { asset_id } => {
                self.handle_reextract_colors(tx, &asset_id).await
            }
            LedgerCommand::UpdateTechnicalMetadata(payload) => {
                crate::infra::database::handlers::metadata_handler::handle_update_technical_metadata(tx, payload).await
            }
            LedgerCommand::UpdateFormat { asset_id, format } => {
                crate::infra::database::handlers::metadata_handler::handle_update_format(tx, &asset_id, &format).await
            }

            // ── Tag CRUD Handlers ──────────────────────────────────────────
            LedgerCommand::CreateTag(payload) => {
                crate::infra::database::handlers::tags_handler::handle_create_tag(tx, payload).await
            }
            LedgerCommand::UpdateTag(payload) => {
                crate::infra::database::handlers::tags_handler::handle_update_tag(tx, payload).await
            }
            LedgerCommand::DeleteTag { id } => {
                crate::infra::database::handlers::tags_handler::handle_delete_tag(tx, id).await
            }
            LedgerCommand::AddTagsToAssetsBatch(payload) => {
                crate::infra::database::handlers::tags_handler::handle_add_tags_to_assets_batch(tx, payload).await
            }
            LedgerCommand::RemoveTagsFromAssetsBatch(payload) => {
                crate::infra::database::handlers::tags_handler::handle_remove_tags_from_assets_batch(tx, payload).await
            }
            LedgerCommand::ReplaceTagsForAssetsBatch(payload) => {
                crate::infra::database::handlers::tags_handler::handle_replace_tags_for_assets_batch(tx, payload).await
            }
            LedgerCommand::CreateSmartFolder(payload) => {
                crate::infra::database::handlers::smart_folder_handler::handle_create_smart_folder(tx, payload).await
            }
            LedgerCommand::UpdateSmartFolder(payload) => {
                crate::infra::database::handlers::smart_folder_handler::handle_update_smart_folder(tx, payload).await
            }
            LedgerCommand::DeleteSmartFolder(payload) => {
                crate::infra::database::handlers::smart_folder_handler::handle_delete_smart_folder(tx, payload).await
            }
            LedgerCommand::RegenerateThumbnail { asset_id } => {
                crate::infra::database::handlers::thumbnail_handler::handle_regenerate_thumbnail(tx, &asset_id).await
            }
            LedgerCommand::Batch(_) => Err(AppError::Internal(
                "Nested Batch commands are not supported".to_string(),
            )),
            LedgerCommand::RenameFolder(payload) => {
                crate::infra::database::handlers::folder_handler::handle_rename_folder(tx, payload).await
            }
        }
    }

    /// Robustly resolves an asset ID from a path.
    pub(crate) async fn resolve_asset_id_robust(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        path: &std::path::Path,
    ) -> AppResult<Option<String>> {
        let path_str = path.to_string_lossy().to_string();

        // 1. Direct Match (NFC/NFD sensitive)
        let row = sqlx::query!(
            r#"SELECT id as "id!" FROM assets WHERE path = ? COLLATE NOCASE"#,
            path_str
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(r) = row {
            return Ok(Some(r.id));
        }

        info!("Ledger: Direct path match FAILED for '{}'. Attempting robust folder+name resolution...", path_str);

        // 2. Fallback: Resolve Folder first, then Name
        if let (Some(parent_path), Some(name)) = (path.parent(), path.file_name()) {
            let name_str = name.to_string_lossy().to_string();

            if let Some(folder_id) = Self::resolve_folder_id_robust(tx, parent_path).await? {
                // Now find the asset by name in this folder
                let asset_row = sqlx::query!(
                    r#"SELECT id as "id!" FROM assets WHERE folder_id = ? AND name = ? COLLATE NOCASE"#,
                    folder_id,
                    name_str
                )
                .fetch_optional(&mut **tx)
                .await?;

                if let Some(a) = asset_row {
                    info!("Ledger: Robust resolution SUCCESS. Found asset ID {} via folder {} + name '{}'", a.id, folder_id, name_str);
                    return Ok(Some(a.id));
                } else {
                    warn!("Ledger: Robust resolution FAILED: Asset '{}' not found in folder ID {} (Normalization mismatch?)", name_str, folder_id);
                }
            } else {
                warn!(
                    "Ledger: Robust resolution FAILED: Could not resolve folder ID for path '{}'",
                    parent_path.display()
                );
            }
        }

        Ok(None)
    }

    /// Robustly resolves a folder ID by path, handling normalization issues.
    pub(crate) async fn resolve_folder_id_robust(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        path: &std::path::Path,
    ) -> AppResult<Option<String>> {
        let path_str = path.to_string_lossy().to_string();

        // 1. Direct match
        let row = sqlx::query!(
            r#"SELECT id as "id!" FROM folders WHERE path = ? COLLATE NOCASE"#,
            path_str
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(r) = row {
            return Ok(Some(r.id));
        }

        // 2. Fallback: Parent ID + Name match
        if let (Some(parent_p), Some(name)) = (path.parent(), path.file_name()) {
            let name_str = name.to_string_lossy().to_string();

            // Recurse to find parent ID
            if let Some(parent_id) = Box::pin(Self::resolve_folder_id_robust(tx, parent_p)).await? {
                let folder_row = sqlx::query!(
                    r#"SELECT id as "id!" FROM folders WHERE parent_id = ? AND name = ? COLLATE NOCASE"#,
                    parent_id,
                    name_str
                )
                .fetch_optional(&mut **tx)
                .await?;

                if let Some(f) = folder_row {
                    info!(
                        "Ledger: Robust folder resolution SUCCESS. Resolved '{}' -> ID {}",
                        path_str, f.id
                    );
                    return Ok(Some(f.id));
                }
            }
        }

        Ok(None)
    }

    /// Performs a one-time database-wide path normalization to NFC.
    /// This resolves legacy "ghost records" on macOS where paths were stored in NFD.
    pub async fn normalize_database_paths(&self) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        info!("Ledger: Starting database path normalization (NFC)...");

        // 1. Normalize Assets
        let asset_rows = sqlx::query!(r#"SELECT id, path FROM assets"#)
            .fetch_all(&mut *tx)
            .await?;

        let mut asset_fix_count = 0;
        for row in asset_rows {
            let nfc_path = row.path.nfc().collect::<String>();
            if nfc_path != row.path {
                sqlx::query!(
                    "UPDATE assets SET path = ?, updated_at = ? WHERE id = ?",
                    nfc_path,
                    now,
                    row.id
                )
                .execute(&mut *tx)
                .await?;
                asset_fix_count += 1;
            }
        }

        // 2. Normalize Folders
        let folder_rows = sqlx::query!(r#"SELECT id, path FROM folders"#)
            .fetch_all(&mut *tx)
            .await?;

        let mut folder_fix_count = 0;
        for row in folder_rows {
            let nfc_path = row.path.nfc().collect::<String>();
            if nfc_path != row.path {
                sqlx::query!(
                    "UPDATE folders SET path = ?, updated_at = ? WHERE id = ?",
                    nfc_path,
                    now,
                    row.id
                )
                .execute(&mut *tx)
                .await?;
                folder_fix_count += 1;
            }
        }

        tx.commit().await?;

        if asset_fix_count > 0 || folder_fix_count > 0 {
            info!(
                "Ledger: Path normalization COMPLETED. Fixed {} assets and {} folders.",
                asset_fix_count, folder_fix_count
            );
        } else {
            info!("Ledger: Database is already normalized.");
        }

        Ok(())
    }
}
