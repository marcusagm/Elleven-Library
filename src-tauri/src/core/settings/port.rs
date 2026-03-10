use super::model::AppSettings;
use crate::core::error::AppResult;
use async_trait::async_trait;

/// Port for persisting and retrieving application settings.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Loads settings from the persistent storage.
    /// If storage doesn't exist, returns default settings.
    ///
    /// # Returns
    ///
    /// Default settings for the application.
    async fn load(&self) -> AppResult<AppSettings>;

    /// Persists settings to the storage.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings to persist.
    ///
    /// # Returns
    ///
    /// Ok(()) if settings were persisted successfully.
    async fn save(&self, settings: &AppSettings) -> AppResult<()>;
}
