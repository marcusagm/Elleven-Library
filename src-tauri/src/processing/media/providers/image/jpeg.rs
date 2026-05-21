use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for JPEG image files (.jpg, .jpeg, .jpe, .jfif).
///
/// Handles all standard JPEG variants. `.jpe` and `.jfif` are treated as aliases
/// for the same format and share extraction logic. Uses `zune-jpeg` for fast
/// decoding and `rexif` for EXIF metadata including resolution (DPI).
///
/// # Technical Details
///
/// - **File Format**: JPEG / JFIF
/// - **Thumbnail Format**: WebP image
/// - **Metadata**: Dimensions, DPI, and full EXIF camera data
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::image::jpeg::JpegFormatProvider;
///
/// let provider = JpegFormatProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats[0].name, "JPEG Image");
/// assert_eq!(formats[0].extensions, vec!["jpg", "jpeg", "jpe", "jfif"]);
/// ```
#[derive(Default)]
pub struct JpegFormatProvider;

impl JpegFormatProvider {
    /// Creates a new instance of `JpegFormatProvider`.
    ///
    /// # Returns
    ///
    /// `JpegFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for JpegFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "JPEG_IMAGE_PROVIDER"
    }

    /// Returns the file extensions handled by this provider.
    ///
    /// `.jpe` and `.jfif` are aliases for the JPEG format.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["jpg", "jpeg", "jpe", "jfif"]
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
            "JPEG Image",
            vec!["jpg", "jpeg", "jpe", "jfif"],
            vec!["image/jpeg"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::None,
        )]
    }

    /// Validates the JPEG magic bytes (`FF D8 FF`).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check (must start with `FF D8 FF`).
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match JPEG format, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"\xFF\xD8\xFF")
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
impl MetadataCapability for JpegFormatProvider {
    /// Extracts width, height, DPI, and EXIF camera metadata from a JPEG file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JPEG file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object with technical metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be opened.
    /// * `AppError::Generic` - If dimension extraction fails.
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

    /// Extracts semantic metadata from a JPEG file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the JPEG file (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - An empty JSON object.
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object as semantic extraction
    /// is not implemented for JPEG files.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for JpegFormatProvider {
    /// Generates a WebP thumbnail from a JPEG file using `zune-jpeg` for fast decoding.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JPEG file.
    /// * `size_hint` - Requested maximum dimension in pixels.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
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
impl PreviewCapability for JpegFormatProvider {
    /// Generates a raster preview from a JPEG file using `image-rs`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JPEG file.
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
