//! Domain models and ports for application configuration.

pub mod model;
pub mod port;

pub use model::{AppLanguage, AppSettings};
pub use port::SettingsRepository;
