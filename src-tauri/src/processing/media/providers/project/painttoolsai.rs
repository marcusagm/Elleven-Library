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

/// Provider for PaintTool SAI (.sai and .sai2) project files.
///
/// This provider handles extraction for PaintTool SAI formats.
/// It extracts technical metadata and thumbnails/previews using specialized internal extractors.
///
/// # Technical Details
///
/// ## File Formats
///
/// - **SAI v1**: Proprietary binary format (`.sai`)
/// - **SAI v2**: Proprietary chunk-based binary format (`.sai2`)
///
/// ## Magic Bytes
///
/// - `.sai`: `SAI`
/// - `.sai2`: `SAI-CANVAS-TYPE0`
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::painttoolsai::PaintToolSaiFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
///
/// let provider = PaintToolSaiFormatProvider::new();
/// let supported_formats = provider.supported_formats();
///
/// assert_eq!(supported_formats.len(), 2);
/// assert_eq!(provider.name(), "PAINTTOOLSAI_PROVIDER");
/// assert_eq!(provider.supported_extensions(), vec!["sai", "sai2"]);
/// ```
#[derive(Default)]
pub struct PaintToolSaiFormatProvider;

impl PaintToolSaiFormatProvider {
    /// Creates a new instance of the PaintTool SAI format provider.
    ///
    /// # Returns
    ///
    /// `PaintToolSaiFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for PaintToolSaiFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "PAINTTOOLSAI_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["sai", "sai2"]
    }

    /// Returns the list of supported formats.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![
            SupportedFormat::with_metadata(
                "PaintTool SAI v1",
                vec!["sai"],
                vec!["application/x-sai"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "PaintTool SAI v2",
                vec!["sai2"],
                vec!["application/x-sai2"],
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
        header_bytes.starts_with(b"SAI") || header_bytes.starts_with(b"SAI-CANVAS-TYPE0")
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
impl ThumbnailCapability for PaintToolSaiFormatProvider {
    /// Generates a thumbnail from the PaintTool SAI file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PaintTool SAI file.
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
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        tokio::task::spawn_blocking(move || {
            if extension == "sai2" {
                let (preview_data, _) = extractors::extract_sai2_preview(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                
                Ok(preview_data)
            } else {
                let (preview_data, _) = extractors::extract_sai_preview(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                
                Ok(preview_data)
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for PaintToolSaiFormatProvider {
    /// Generates a preview from the PaintTool SAI file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PaintTool SAI file.
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
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        tokio::task::spawn_blocking(move || {
            if extension == "sai2" {
                extractors::extract_sai2_preview(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
            } else {
                extractors::extract_sai_preview(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for PaintToolSaiFormatProvider {
    /// Extracts technical metadata from the PaintTool SAI file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PaintTool SAI file.
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
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        tokio::task::spawn_blocking(move || {
            if extension == "sai2" {
                let metadata = extractors::extract_sai2_metadata(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                Ok(metadata["technical"].clone())
            } else {
                let metadata = extractors::extract_sai_metadata(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                Ok(metadata["technical"].clone())
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the PaintTool SAI file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PaintTool SAI file.
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
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        tokio::task::spawn_blocking(move || {
            if extension == "sai2" {
                let metadata = extractors::extract_sai2_metadata(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                Ok(metadata["semantic"].clone())
            } else {
                let metadata = extractors::extract_sai_metadata(&path_owned)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
                Ok(metadata["semantic"].clone())
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
