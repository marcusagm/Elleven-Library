use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};
use crate::core::events::{AppEventBus, DomainEvent};
use crate::core::ledger::command::LedgerCommand;
use crate::core::ledger::port::TransactionalAssetLedger;
use crate::core::models::asset::{Asset, AssetState};

/// SQLite implementation of the Asset Ledger using SQLx.
///
/// This adapter ensures that all mutations are atomic and audited
/// via the `v2_asset_operations_log` table before publishing domain events.
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
            INSERT INTO v2_asset_operations_log (id, operation_type, asset_id, payload, status, error_note, created_at)
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
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated asset or an error.
    async fn execute(&self, command: LedgerCommand) -> AppResult<Asset> {
        let mut tx = self.pool.begin().await?;

        let result = match &command {
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

                // 1. Insert Asset
                sqlx::query!(
                    r#"
                    INSERT INTO v2_assets (id, name, path, state, format_type, family, file_size, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                    asset_id,
                    name,
                    path_str,
                    state_str,
                    payload.format_type,
                    payload.family,
                    file_size_i64,
                    now,
                    now
                )
                .execute(&mut *tx)
                .await?;

                // 2. Audit Log
                let op_payload = serde_json::to_value(payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    &mut tx,
                    "CREATE_ASSET",
                    &asset_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                let asset = Asset {
                    id: asset_id,
                    name: name.to_string(),
                    path: payload.path.clone(),
                    state: payload.state_init,
                    format_type: payload.format_type.clone(),
                    family: payload.family.clone(),
                    file_size: payload.file_size,
                    created_at: Some(now),
                    updated_at: Some(now),
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

                    // 1. Insert Asset
                    sqlx::query!(
                        r#"
                        INSERT INTO v2_assets (id, name, path, state, format_type, family, file_size, created_at, updated_at)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                        "#,
                        asset_id,
                        name,
                        path_str,
                        state_str,
                        payload.format_type,
                        payload.family,
                        file_size_i64,
                        now,
                        now
                    )
                    .execute(&mut *tx)
                    .await?;

                    // 2. Audit Log
                    let op_payload = serde_json::to_value(payload).map_err(|e| {
                        AppError::Internal(format!("Failed to serialize payload: {}", e))
                    })?;

                    Self::log_operation(
                        &mut tx,
                        "CREATE_ASSET_BATCH_MEMBER",
                        &asset_id,
                        op_payload,
                        "COMPLETED",
                        None,
                    )
                    .await?;

                    created_assets.push(Asset {
                        id: asset_id,
                        name: name.to_string(),
                        path: payload.path.clone(),
                        state: payload.state_init,
                        format_type: payload.format_type.clone(),
                        family: payload.family.clone(),
                        file_size: payload.file_size,
                        created_at: Some(now),
                        updated_at: Some(now),
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
                // Sprint 2.2 focused on Asset/Log atomic mutation.
                // Tag implementation will be fleshed out in future sprints,
                // but we must audit the request and update updated_at.

                let now = Utc::now();
                sqlx::query!(
                    "UPDATE v2_assets SET updated_at = ? WHERE id = ?",
                    now,
                    payload.asset_id
                )
                .execute(&mut *tx)
                .await?;

                let op_payload = serde_json::to_value(payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    &mut tx,
                    "UPDATE_TAGS",
                    &payload.asset_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // Fetch asset to return (Simulation for now as tag logic is pending)
                let row = sqlx::query_as!(
                    crate::infra::database::models::AssetDb,
                    r#"SELECT id as "id!", name as "name!", path as "path!", state as "state!", format_type as "format_type!", family as "family!", file_size as "file_size!", created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>" FROM v2_assets WHERE id = ?"#,
                    payload.asset_id
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound(payload.asset_id.clone()))?;

                Ok(row.into())
            }
            LedgerCommand::UpdateAsset(payload) => {
                let now = Utc::now();

                // 1. Resolve Asset ID
                let asset_id: String = match (&payload.asset_id, &payload.old_path) {
                    (Some(id), _) => id.clone(),
                    (None, Some(old_path)) => {
                        let old_path_str = old_path.to_string_lossy().to_string();
                        let row = sqlx::query!(
                            r#"SELECT id as "id!" FROM v2_assets WHERE path = ?"#,
                            old_path_str
                        )
                        .fetch_optional(&mut *tx)
                        .await?;
                        row.map(|r| r.id).ok_or_else(|| {
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
                    "UPDATE v2_assets SET path = ?, name = ?, updated_at = ? WHERE id = ?",
                    new_path_str,
                    new_name,
                    now,
                    asset_id
                )
                .execute(&mut *tx)
                .await?;

                // 3. Audit Log
                let op_payload = serde_json::to_value(payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize payload: {}", e))
                })?;

                Self::log_operation(
                    &mut tx,
                    "UPDATE_ASSET",
                    &asset_id,
                    op_payload,
                    "COMPLETED",
                    None,
                )
                .await?;

                // 4. Fetch and return
                let row = sqlx::query_as!(
                    crate::infra::database::models::AssetDb,
                    r#"SELECT id as "id!", name as "name!", path as "path!", state as "state!", format_type as "format_type!", family as "family!", file_size as "file_size!", created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>" FROM v2_assets WHERE id = ?"#,
                    asset_id
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound(asset_id))?;

                Ok(row.into())
            }
            LedgerCommand::MarkAsStale { asset_id } => {
                let now = Utc::now();
                let state_stale = AssetState::Stale.to_string();

                sqlx::query!(
                    "UPDATE v2_assets SET state = ?, updated_at = ? WHERE id = ?",
                    state_stale,
                    now,
                    asset_id
                )
                .execute(&mut *tx)
                .await?;

                Self::log_operation(
                    &mut tx,
                    "MARK_STALE",
                    asset_id,
                    serde_json::json!({}),
                    "COMPLETED",
                    None,
                )
                .await?;

                let row = sqlx::query_as!(
                    crate::infra::database::models::AssetDb,
                    r#"SELECT id as "id!", name as "name!", path as "path!", state as "state!", format_type as "format_type!", family as "family!", file_size as "file_size!", created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>" FROM v2_assets WHERE id = ?"#,
                    asset_id
                )
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound(asset_id.clone()))?;

                Ok(row.into())
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
                            r#"SELECT id as "id!" FROM v2_assets WHERE path = ?"#,
                            path_str
                        )
                        .fetch_optional(&mut *tx)
                        .await?;
                        row.map(|r| r.id).ok_or_else(|| {
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
                sqlx::query!("DELETE FROM v2_assets WHERE id = ?", resolved_id)
                    .execute(&mut *tx)
                    .await?;

                // 3. Audit Log
                Self::log_operation(
                    &mut tx,
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
                    updated_at: None,
                })
            }
        };

        match result {
            Ok(asset) => {
                tx.commit().await?;

                // Publish Domain Events only AFTER commit
                match &command {
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
                            active_tags: p.tags_to_add.clone(), // Simplified
                        })?;
                    }
                    LedgerCommand::UpdateAsset(_) => {
                        self.event_bus.publish(DomainEvent::AssetMetadataUpdated {
                            asset_id: asset.id.clone(),
                        })?;
                    }
                    LedgerCommand::DeleteAsset { .. } => {
                        self.event_bus.publish(DomainEvent::FsPathDeleted {
                            path: asset.id.clone(), // Using asset.id (the resolved ID)
                        })?;
                    }
                    _ => {}
                }

                Ok(asset)
            }
            Err(e) => {
                // Transaction is dropped here, effecting a Rollback
                Err(e)
            }
        }
    }
}
