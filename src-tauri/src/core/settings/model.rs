use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported application languages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppLanguage {
    English,
    Portuguese,
}

/// Default language is English.
impl Default for AppLanguage {
    fn default() -> Self {
        Self::English
    }
}

/// Core application configuration that must persist across sessions.
///
/// This structure is the source of truth for UI and Backend behavior
/// and is kept outside the main SQLite database for resiliency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Custom path for the asset database (if any).
    pub database_path: Option<PathBuf>,

    /// Maximum number of parallel worker threads for processing.
    /// 0 means auto-detect (half of available cores).
    pub worker_threads: usize,

    /// Preferred UI language.
    pub ui_language: AppLanguage,

    /// Whether to automatically scan watched directories on boot.
    pub auto_scan_enabled: bool,
}

/// Default settings for the application.
impl Default for AppSettings {
    /// Default settings for the application.
    ///
    /// # Returns
    ///
    /// Default settings for the application.
    fn default() -> Self {
        let available_parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self {
            database_path: None,
            worker_threads: std::cmp::max(1, available_parallelism / 2),
            ui_language: AppLanguage::default(),
            auto_scan_enabled: true,
        }
    }
}
