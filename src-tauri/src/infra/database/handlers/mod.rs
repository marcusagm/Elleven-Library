//! Specialized Command Handlers for the Infra Database Layer.
//!
//! Each sub-module encapsulates all SQL mutations and audit-log entries for a
//! specific domain concept, keeping `SqliteAssetLedger` as a pure transactional
//! router rather than a "God Adapter".
pub mod asset_handler;
pub mod folder_handler;
pub mod tags_handler;
pub mod metadata_handler;
pub mod smart_folder_handler;
pub mod thumbnail_handler;
