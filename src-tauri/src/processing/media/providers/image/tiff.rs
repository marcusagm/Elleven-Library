use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for TIFF image files (.tiff, .tif).
///
/// `.tif` is an alias for `.tiff` and shares the same extraction logic. TIFF is
/// a lossless multi-layer format that is not natively rendered by browsers, so
/// preview uses `Convert` strategy (server-side transcoding to PNG/WebP).
/// Supports DPI extraction via EXIF XResolution/YResolution tags.
///
/// # Technical Details
///
/// - **File Format**: TIFF
/// - **Thumbnail Format**: WebP image
/// - **Metadata**: Dimensions, DPI (via EXIF), and full EXIF data
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::image::tiff::TiffFormatProvider;
///
/// let provider = TiffFormatProvider::new();
/// assert!(provider.supported_extensions().contains(&"tif"));
/// assert!(provider.supported_extensions().contains(&"tiff"));
/// ```
#[derive(Default)]
pub struct TiffFormatProvider;

impl TiffFormatProvider {
    /// Creates a new instance of `TiffFormatProvider`.
    ///
    /// # Returns
    ///
    /// `TiffFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for TiffFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "TIFF_IMAGE_PROVIDER"
    }

    /// Returns both `tiff` and its alias `tif`.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["tiff", "tif"]
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
            "TIFF Image",
            vec!["tiff", "tif"],
            vec!["image/tiff"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::Convert,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the header bytes match the TIFF format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match TIFF format, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"II*\x00") || header_bytes.starts_with(b"MM\x00*")
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
impl MetadataCapability for TiffFormatProvider {
    /// Extracts width, height, DPI, and EXIF data from a TIFF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TIFF file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object with technical metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be opened.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_raster_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from a TIFF file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the TIFF file (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - An empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for TiffFormatProvider {
    /// Generates a WebP thumbnail from a TIFF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TIFF file.
    /// * `_asset_id` - The ID of the asset (unused in current implementation).
    /// * `size_hint` - The desired size of the thumbnail (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the generated thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If decoding or encoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
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
impl PreviewCapability for TiffFormatProvider {
    /// Generates a raster preview from a TIFF file using `image-rs`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TIFF file.
    /// * `_asset_id` - The ID of the asset (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the preview cannot be generated.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
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
