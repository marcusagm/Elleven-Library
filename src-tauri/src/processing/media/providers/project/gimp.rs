use std::path::Path;

use async_trait::async_trait;
use serde_json::Value;
use tracing::instrument;

use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::formats::types::{
    MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
};
use crate::processing::media::extractors;

/// Provider for GIMP XCF (.xcf) files.
///
/// This provider uses internal chunk parsing to extract metadata and uses layer
/// compositing logic to generate previews for XCF files.
///
/// # Technical Details
///
/// - **File Format**: GIMP proprietary binary format (supports v001 to v013)
/// - **Preview Format**: Extracted and composited layers rendered as a PNG image
/// - **Metadata**: Extracted from header and properties (Width, Height, Version, DPI)
/// - **Magic Bytes**: `gimp xcf `
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::gimp::GimpFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
///
/// let provider = GimpFormatProvider::new();
/// let supported_formats = provider.supported_formats();
///
/// assert_eq!(supported_formats.len(), 1);
/// assert_eq!(provider.name(), "GIMP_PROVIDER");
/// assert_eq!(provider.supported_extensions(), vec!["xcf"]);
/// ```
#[derive(Default)]
pub struct GimpFormatProvider;

impl GimpFormatProvider {
    /// Creates a new instance of the GIMP format provider.
    ///
    /// # Returns
    ///
    /// `GimpFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for GimpFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "GIMP_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["xcf"]
    }

    /// Returns the list of supported formats.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![SupportedFormat::with_metadata(
            "GIMP Image",
            vec!["xcf"],
            vec!["image/x-xcf"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the provider supports the given magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if the provider supports the given magic bytes.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"gimp xcf")
    }

    /// Returns the metadata capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Returns the preview capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl ThumbnailCapability for GimpFormatProvider {
    /// Generates a thumbnail from the GIMP file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    /// * `asset_id` - Identifier for the asset.
    /// * `size_hint` - Hint for the desired size.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail data.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let (preview_data, _) = extractors::extract_xcf_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(preview_data)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for GimpFormatProvider {
    /// Generates a preview from the GIMP file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
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
            extractors::extract_xcf_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for GimpFormatProvider {
    /// Extracts technical metadata from the GIMP file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
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
            let metadata = extractors::extract_xcf_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(metadata["technical"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the GIMP file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
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
            let metadata = extractors::extract_xcf_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(metadata["semantic"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
