use std::path::Path;

use async_trait::async_trait;
use serde_json::Value;
use tracing::instrument;

use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};
use crate::processing::media::extractors;

/// Provider for CLIP Studio Paint (.clip) files.
///
/// This provider uses the internal SQLite database chunk (`CHNKSQLi`) embedded in `.clip` files 
/// to extract high-quality metadata (dimensions, resolution) and previews.
///
/// # Technical Details
///
/// - **File Format**: Custom chunk-based binary with an embedded SQLite database
/// - **Preview Format**: PNG image extracted from `CanvasPreview` table
/// - **Metadata**: Extracted from `Canvas` table (Width, Height, Resolution)
/// - **Magic Bytes**: `CSFCHUNK`
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::clipstudio::ClipStudioFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
///
/// let provider = ClipStudioFormatProvider::new();
/// let supported_formats = provider.supported_formats();
///
/// assert_eq!(supported_formats.len(), 1);
/// assert_eq!(provider.name(), "CLIPSTUDIO_PROVIDER");
/// assert_eq!(provider.supported_extensions(), vec!["clip"]);
/// ```
#[derive(Default)]
pub struct ClipStudioFormatProvider;

impl ClipStudioFormatProvider {
    /// Creates a new instance of the CLIP Studio format provider.
    ///
    /// # Returns
    ///
    /// `ClipStudioFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for ClipStudioFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "CLIPSTUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["clip"]
    }

    /// Returns the list of supported formats.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![
            SupportedFormat::with_metadata(
                "CLIP Studio Paint Document",
                vec!["clip"],
                vec!["application/x-clip-studio-paint"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
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
        header_bytes.starts_with(b"CSFCHUNK")
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
impl ThumbnailCapability for ClipStudioFormatProvider {
    /// Generates a thumbnail from the CLIP Studio file.
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
            let (preview_data, _) = extractors::extract_clip_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(preview_data)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for ClipStudioFormatProvider {
    /// Generates a preview from the CLIP Studio file.
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
            extractors::extract_clip_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for ClipStudioFormatProvider {
    /// Extracts technical metadata from the CLIP Studio file.
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
            let metadata = extractors::extract_clip_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(metadata["technical"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the CLIP Studio file.
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
            let metadata = extractors::extract_clip_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(metadata["semantic"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
