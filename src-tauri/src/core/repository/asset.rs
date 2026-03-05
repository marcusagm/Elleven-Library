use crate::core::error::AppResult;
use crate::core::models::Asset;
use async_trait::async_trait;

/// Port for read-only asset operations.
#[async_trait]
pub trait AssetQueryHandler: Send + Sync {
    /// Retrieves a list of all assets in the library.
    ///
    /// # Errors
    /// Returns `AppError` if the underlying storage fails.
    async fn find_all(&self) -> AppResult<Vec<Asset>>;
}
