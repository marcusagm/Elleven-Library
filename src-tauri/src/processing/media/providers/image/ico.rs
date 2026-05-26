use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Windows Icon files (.ico).
///
/// Handles standard uncompressed ICO files. Preview is served directly by the
/// browser. Thumbnails are generated via `image-rs`.
///
/// # Technical Details
///
/// - **File Format**: ICO
/// - **Preview Format**: PNG image
/// - **Thumbnail Format**: PNG image
///
/// # Example
///
/// ```no_run
/// use mundam::core::formats::provider::FormatProvider;
/// use mundam::core::formats::types::PreviewStrategy;
/// use mundam::processing::media::providers::image::IcoFormatProvider;
/// use mundam::core::formats::capabilities::PreviewCapability;
/// use std::path::Path;
///
/// async fn generate_ico_preview(path: &Path) -> mundam::core::AppResult<(Vec<u8>, String)> {
///     let provider = IcoFormatProvider;
///     provider.generate_preview(path, "asset_id").await
/// }
/// ```
#[derive(Default)]
pub struct IcoFormatProvider;

impl IcoFormatProvider {
    /// Creates a new instance of `IcoFormatProvider`.
    ///
    /// # Returns
    ///
    /// `IcoFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for IcoFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "ICO_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ico"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats with metadata.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "Windows Icon",
            vec!["ico"],
            vec!["image/x-icon", "image/vnd.microsoft.icon"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the header bytes match the ICO format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match ICO format, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(&[0, 0, 1, 0])
    }

    /// Returns the preview capability for this provider.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }

    /// Returns the metadata capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata capability for this provider.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail capability for this provider.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for IcoFormatProvider {
    /// Extracts width and height from an ICO file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ICO file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object with technical metadata.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_raster_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from an ICO file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the ICO file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object with semantic metadata.
    #[instrument(skip(self, _path))]
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for IcoFormatProvider {
    /// Generates a WebP thumbnail from an ICO file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ICO file.
    /// * `_asset_id` - The ID of the asset.
    /// * `size_hint` - The desired size of the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the thumbnail data.
    ///
    /// # Error
    ///
    /// `AppError::ExtractionProcessTimeout` - If the extraction process times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_raster_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for IcoFormatProvider {
    /// Generates a preview from an ICO file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ICO file.
    /// * `_asset_id` - The ID of the asset.
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and its mime type.
    ///
    /// # Error
    ///
    /// `AppError::ExtractionProcessTimeout` - If the extraction process times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_raster_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
