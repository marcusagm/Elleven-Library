use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Windows Cursor files (.cur).
///
/// Handles standard uncompressed CUR files. Preview is served directly by the
/// browser. Thumbnails are generated via `image-rs`.
///
/// # Technical Details
/// 
/// - **File Format**: PSD or PSB
/// - **Preview Format**: PNG image
/// - **Metadata**: JSON data containing design information
///
/// - The CUR format is a bitmap image format used for displaying cursors in Windows.
/// - It supports transparency and hot spots.
/// - It is a container format that can hold multiple cursor images of different sizes and color depths.
///
/// # Example
///
/// ```no_run
/// use mundam::core::formats::provider::FormatProvider;
/// use mundam::core::formats::types::ThumbnailStrategy;
/// use mundam::processing::media::providers::image::CurFormatProvider;
/// use mundam::core::formats::capabilities::ThumbnailCapability;
/// use std::path::Path;
///
/// async fn generate_cur_thumbnail(path: &Path) -> mundam::core::AppResult<Vec<u8>> {
///     let provider = CurFormatProvider;
///     provider.generate(path, "asset_id", 256).await
/// }
/// ```
#[derive(Default)]
pub struct CurFormatProvider;

impl CurFormatProvider {
    /// Creates a new instance of `CurFormatProvider`.
    ///
    /// # Returns
    ///
    /// `CurFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for CurFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "CUR_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cur"]
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
            "Windows Cursor",
            vec!["cur"],
            vec!["image/x-icon"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the header bytes match the CUR format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match CUR format, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(&[0, 0, 2, 0])
    }

    /// Returns the preview capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview capability for this provider.
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
impl MetadataCapability for CurFormatProvider {
    /// Extracts width and height from a CUR file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CUR file.
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

    /// Extracts semantic metadata from a CUR file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the CUR file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object with semantic metadata.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for CurFormatProvider {
    /// Generates a WebP thumbnail from a CUR file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CUR file.
    /// * `_asset_id` - The ID of the asset.
    /// * `size_hint` - The desired size of the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the thumbnail image.
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
impl PreviewCapability for CurFormatProvider {
    /// Generates a preview image from a CUR file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CUR file.
    /// * `_asset_id` - The ID of the asset.
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview image as bytes and its format.
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
