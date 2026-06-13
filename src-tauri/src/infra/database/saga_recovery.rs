use crate::core::error::{AppError, AppResult};
use sqlx::SqlitePool;

use tokio::fs;
use tracing::{info, warn};

/// Service responsible for recovering and completing pending Sagas (Outbox pattern).
/// 
/// This guarantees atomicity between database transactions and filesystem operations.
/// If a process crashes after a database commit but before the filesystem operation
/// completes, this service will pick up the `PENDING` operations on startup
/// and either retry them or execute compensating actions.
pub struct SagaRecoveryService {
    pool: SqlitePool,
}

impl SagaRecoveryService {
    /// Creates a new instance of the SagaRecoveryService.
    ///
    /// # Arguments
    ///
    /// * `pool` - The SQLite database connection pool.
    ///
    /// # Returns
    ///
    /// A new instance of `SagaRecoveryService`.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Runs the recovery process. Should be called during application startup.
    ///
    /// # Errors
    ///
    /// Returns `AppError` if database queries fail or if there are critical
    /// issues executing the pending operations.
    pub async fn run_recovery(&self) -> AppResult<()> {
        info!("SagaRecovery: Starting recovery of pending operations...");

        // Find all pending operations ordered by oldest first
        let pending_operations = sqlx::query!(
            r#"
            SELECT 
                id as "id!", 
                operation_type as "operation_type!", 
                asset_id as "asset_id!", 
                payload as "payload!", 
                status, 
                error_note, 
                created_at
            FROM asset_operations_log
            WHERE status = 'PENDING'
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        if pending_operations.is_empty() {
            info!("SagaRecovery: No pending operations found. System is clean.");
            return Ok(());
        }

        info!("SagaRecovery: Found {} pending operations. Processing...", pending_operations.len());

        for operation in pending_operations {
            match self.process_operation(&operation.id, &operation.operation_type, &operation.asset_id, &operation.payload).await {
                Ok(_) => {
                    info!("SagaRecovery: Operation {} ({}) completed successfully.", operation.id, operation.operation_type);
                    self.mark_completed(&operation.id).await?;
                }
                Err(e) => {
                    warn!("SagaRecovery: Failed to process operation {} ({}): {}", operation.id, operation.operation_type, e);
                    self.mark_failed(&operation.id, &e.to_string()).await?;
                }
            }
        }

        info!("SagaRecovery: Recovery process finished.");
        Ok(())
    }

    /// Processes a single pending saga operation.
    ///
    /// # Arguments
    ///
    /// * `operation_id` - The unique identifier of the operation log.
    /// * `operation_type` - The string representing the operation type (e.g., "DELETE_ASSET").
    /// * `_asset_id` - The unique identifier of the asset associated with the operation.
    /// * `payload_string` - The JSON payload string containing operation context.
    ///
    /// # Errors
    ///
    /// Returns `AppError` if payload parsing fails or if the actual recovery steps fail.
    async fn process_operation(
        &self,
        operation_id: &str,
        operation_type: &str,
        _asset_id: &str,
        payload_string: &str,
    ) -> AppResult<()> {
        let payload: serde_json::Value = serde_json::from_str(payload_string).map_err(|error| {
            AppError::Internal(format!("Failed to parse payload for operation {}: {}", operation_id, error))
        })?;

        match operation_type {
            "DELETE_ASSET" => {
                let physical_delete = payload.get("physical").and_then(|value| value.as_bool()).unwrap_or(false);
                let path_string = payload.get("path").and_then(|value| value.as_str());

                if physical_delete {
                    if let Some(path_reference) = path_string {
                        let path_buffer = std::path::PathBuf::from(path_reference);
                        if path_buffer.exists() {
                            info!("SagaRecovery: Executing pending physical delete for: {}", path_reference);
                            if let Err(error) = fs::remove_file(&path_buffer).await {
                                warn!("SagaRecovery: Failed to delete file {}: {}", path_reference, error);
                                // If it fails because of permission or lock, we might want to return Error so it stays PENDING.
                                // For now, we return error so it goes to FAILED.
                                return Err(AppError::Io(error));
                            }
                        } else {
                            info!("SagaRecovery: File {} already deleted.", path_reference);
                        }
                    }
                }
            }
            // Add other operations here (RenameFolder, CreateAsset, etc.)
            _ => {
                info!("SagaRecovery: Operation type '{}' requires no recovery or is not implemented yet.", operation_type);
            }
        }

        Ok(())
    }

    /// Marks a saga operation as successfully completed.
    ///
    /// # Arguments
    ///
    /// * `operation_id` - The unique identifier of the operation log.
    ///
    /// # Errors
    ///
    /// Returns `AppError` if the database update fails.
    async fn mark_completed(&self, operation_id: &str) -> AppResult<()> {
        sqlx::query!(
            "UPDATE asset_operations_log SET status = 'COMPLETED', error_note = NULL WHERE id = ?",
            operation_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Marks a saga operation as failed.
    ///
    /// # Arguments
    ///
    /// * `operation_id` - The unique identifier of the operation log.
    /// * `error_message` - The error string that caused the failure.
    ///
    /// # Errors
    ///
    /// Returns `AppError` if the database update fails.
    async fn mark_failed(&self, operation_id: &str, error_message: &str) -> AppResult<()> {
        sqlx::query!(
            "UPDATE asset_operations_log SET status = 'FAILED', error_note = ? WHERE id = ?",
            error_message,
            operation_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
