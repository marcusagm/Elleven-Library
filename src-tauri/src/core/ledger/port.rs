use super::command::LedgerCommand;
use crate::core::error::AppResult;
use crate::core::models::asset::Asset;
use async_trait::async_trait;

/// Port (Interface) for the Transactional Asset Ledger.
///
/// The Ledger is the Single Source of Truth for all state mutations.
/// It ensures atomicity between database updates and filesystem operations.
#[async_trait]
pub trait TransactionalAssetLedger: Send + Sync {
    /// Executes a mutation command and returns the resulting Asset entity.
    ///
    /// # Arguments
    /// * `command` - The mutation intent (Create, Update, Delete, etc).
    ///
    /// # Errors
    /// Returns `AppError` if validation fails, a state transition is illegal,
    /// or if the underlying storage/filesystem operation fails.
    async fn execute(&self, command: LedgerCommand) -> AppResult<Asset>;
}
