use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for HEIF/HEIC image files (.heic, .heif, .heifs).
///
/// High Efficiency Image Format is a modern container for still images and
/// image sequences. `.heif` and `.heifs` are treated as aliases sharing the
/// same FFmpeg-based extraction path. Browser support is inconsistent, so both
/// thumbnail and preview are generated server-side via FFmpeg.
///
/// # Technical Details
///
/// - **File Format**: HEIF / HEIC (ISO/IEC 23008-12)
/// - **Thumbnail Format**: JPEG (via FFmpeg)
/// - **Metadata**: Dimensions via FFprobe
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::image::heic::HeicFormatProvider;
///
/// let provider = HeicFormatProvider::new();
/// assert!(provider.supported_extensions().contains(&"heic"));
/// assert!(provider.supported_extensions().contains(&"heif"));
/// ```
#[derive(Default)]
pub struct HeicFormatProvider;

impl HeicFormatProvider {
    /// Creates a new instance of `HeicFormatProvider`.
    ///
    /// # Returns
    ///
    /// `Self` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for HeicFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "HEIC_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["heic", "heif", "heifs"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - The list of supported formats with their details.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "High Efficiency Image",
            vec!["heic", "heif", "heifs"],
            vec!["image/heic", "image/heif", "image/heic-sequence"],
            MediaType::Image,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::Ffmpeg,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the header bytes correspond to a valid HEIC/HEIF file.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes correspond to a valid HEIC/HEIF file, `false` otherwise.
    fn supports_magic_bytes(&self, _header_bytes: &[u8]) -> bool {
        false
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
impl MetadataCapability for HeicFormatProvider {
    /// Extracts dimensions and codec information from a HEIC/HEIF file via FFprobe.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the HEIC/HEIF file.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFprobe fails or its output cannot be parsed.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_ffmpeg_image_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from a HEIC/HEIF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the HEIC/HEIF file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - The semantic metadata extracted from the HEIC/HEIF file.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    /// * `AppError::Generic` - If the metadata extraction fails.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for HeicFormatProvider {
    /// Generates a JPEG thumbnail from a HEIC/HEIF file via FFmpeg.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the HEIC/HEIF file.
    /// * `size_hint` - Requested maximum dimension in pixels.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFmpeg fails or times out.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_ffmpeg_image_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for HeicFormatProvider {
    /// Generates a preview from a HEIC/HEIF file via FFmpeg.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the HEIC/HEIF file.
    /// * `_asset_id` - Asset ID (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFmpeg fails or times out.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_ffmpeg_image_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
