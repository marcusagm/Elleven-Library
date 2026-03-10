use crate::core::error::AppResult;
use crate::core::settings::{AppSettings, SettingsRepository};
use std::sync::Arc;

/// Application service for managing settings.
/// Bridges the UI requirements with the domain and infrastructure.
pub struct SettingsService {
    repository: Arc<dyn SettingsRepository>,
}

/// Implementation of SettingsService.
impl SettingsService {
    /// Creates a new settings service.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository for managing settings.
    ///
    /// # Returns
    ///
    /// New SettingsService instance.
    pub fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }

    /// Loads the current application settings.
    ///
    /// # Returns
    ///
    /// Default settings for the application.
    pub async fn get_settings(&self) -> AppResult<AppSettings> {
        self.repository.load().await
    }

    /// Updates and persists application settings.
    ///
    /// # Arguments
    ///
    /// * `settings` - Settings to persist.
    ///
    /// # Returns
    ///
    /// Ok(()) if settings were persisted successfully.
    pub async fn update_settings(&self, settings: AppSettings) -> AppResult<()> {
        self.repository.save(&settings).await
    }
}
