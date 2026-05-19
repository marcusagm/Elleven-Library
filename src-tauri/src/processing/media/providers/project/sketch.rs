//! Sketch (.sketch) format provider.
//!
//! Provides metadata, thumbnail, and preview extraction for Sketch files.
//! Leverages the internal ZIP structure and `meta.json`.

use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::formats::types::{
    MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
};
use crate::core::AppResult;
use crate::processing::media::extractors;
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::instrument;

/// Provider for Sketch (.sketch) project files.
///
/// Provides metadata, thumbnail, and preview extraction for Sketch files.
/// Leverages the internal ZIP structure and `meta.json`.
///
/// # Technical Details
///
/// - **File Format**: ZIP archive
/// - **Preview Format**: PNG image (first page)
/// - **Metadata**: JSON data containing design information
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::sketch::SketchFormatProvider;
///
/// let provider = SketchFormatProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].name, "Sketch Project");
/// assert_eq!(formats[0].extensions, vec!["sketch"]);
/// ```
#[derive(Default)]
pub struct SketchFormatProvider;

impl SketchFormatProvider {
    /// Creates a new instance of the `SketchFormatProvider`.
    ///
    /// # Returns
    ///
    /// `SketchFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for SketchFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "SKETCH_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["sketch"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats with metadata.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![SupportedFormat::with_metadata(
            "Sketch Project",
            vec!["sketch"],
            vec!["application/x-sketch"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches the Sketch magic bytes (ZIP header).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid Sketch file (ZIP container).
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // ZIP container: PK..
        header_bytes.starts_with(b"PK\x03\x04")
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl ThumbnailCapability for SketchFormatProvider {
    /// Generates a thumbnail from the Sketch file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Sketch file.
    /// * `asset_id` - Identifier for the asset.
    /// * `size_hint` - Hint for the desired size.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail data.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the thumbnail extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_sketch_preview(&path_owned)
                .map(|(data, _)| data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for SketchFormatProvider {
    /// Generates a preview from the Sketch file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Sketch file.
    /// * `asset_id` - Identifier for the asset.
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - Preview data and format.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the preview extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_sketch_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for SketchFormatProvider {
    /// Extracts technical metadata from the Sketch file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Sketch file.
    ///
    /// # Returns
    ///
    /// `AppResult<Value>` - Technical metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the metadata extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_sketch_metadata(&path_owned)
                .map(|meta| meta["technical"].clone())
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the Sketch file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Sketch file.
    ///
    /// # Returns
    ///
    /// `AppResult<Value>` - Semantic metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the metadata extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_semantic(&self, path: &Path) -> AppResult<Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_sketch_metadata(&path_owned)
                .map(|meta| meta["semantic"].clone())
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
