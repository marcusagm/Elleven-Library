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
    /// * `size_hint` - A hint for the requested thumbnail size (e.g., width in pixels).
    ///
    /// # Errors
    /// Returns `AppError` if generation fails.
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>>;
}
