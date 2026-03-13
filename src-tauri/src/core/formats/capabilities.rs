use crate::core::error::AppResult;
use async_trait::async_trait;
use std::path::Path;

/// A Pure Capability (A Hexagonal abstract "Port" with only IO contracts).
///
/// This trait defines the ability to extract metadata from a file.
#[async_trait]
pub trait MetadataCapability: Send + Sync {
    /// Extracts technical metadata: width, height, bitrate, focal length, exif, etc.
    ///
    /// # Arguments
    /// * `path` - The path to the file on disk.
    ///
    /// # Errors
    /// Returns `AppError` if extraction fails.
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value>;

    /// Extracts semantic metadata: NLP text, OCR, embedded AI tags, etc.
    ///
    /// # Arguments
    /// * `path` - The path to the file on disk.
    ///
    /// # Errors
    /// Returns `AppError` if extraction fails.
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value>;
}

/// The Capability for visual "photography" of the FileSystem.
///
/// This trait defines the ability to generate thumbnails for a file.
#[async_trait]
pub trait ThumbnailCapability: Send + Sync {
    /// Generates a thumbnail for the file at the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file on disk.
    /// * `asset_id` - The unique identifier for the asset.
    /// * `size_hint` - A hint for the requested thumbnail size (e.g., width in pixels).
    ///
    /// # Errors
    /// Returns `AppError` if generation fails.
    async fn generate(&self, path: &Path, asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>>;
}

/// The Capability for high-resolution preview extraction.
///
/// This is used for formats that cannot be rendered natively by the browser
/// but contain a high-res preview (e.g. RAW, Krita, PSD).
#[async_trait]
pub trait PreviewCapability: Send + Sync {
    /// Generates/Extracts a high-resolution preview for the file.
    ///
    /// # Arguments
    /// * `path` - The path to the file on disk.
    /// * `asset_id` - The unique identifier for the asset.
    ///
    /// # Returns
    /// A pair of (bytes, mime_type).
    ///
    /// # Errors
    /// Returns `AppError` if generation fails.
    async fn generate_preview(&self, path: &Path, asset_id: &str) -> AppResult<(Vec<u8>, String)>;
}
