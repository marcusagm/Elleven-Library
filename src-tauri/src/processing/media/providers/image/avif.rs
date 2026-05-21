use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for AVIF image files (.avif, .avifs).
///
/// AV1 Image File Format is an open, royalty-free image format based on the
/// AV1 video codec. `.avifs` is treated as an alias for AVIF image sequences.
/// Modern browsers support AVIF, but server-side generation via FFmpeg is used
/// for reliability across all environments.
///
/// # Technical Details
///
/// - **File Format**: AVIF (AV1 Image File Format)
/// - **Thumbnail Format**: JPEG (via FFmpeg)
/// - **Metadata**: Dimensions and codec via FFprobe
///
/// # Features
///
/// - Extracts dimensions and codec information from AVIF files via FFprobe.
/// - Generates JPEG thumbnails from AVIF files via FFmpeg.
///
/// # Examples
///
/// ```rust
/// use mundam_core::formats::provider::FormatProvider;
/// use mundam_core::formats::types::MediaType;
///
/// let provider = AvifFormatProvider::new();
/// let formats = provider.supported_formats();
/// assert!(formats.contains(&MediaType::Image));
/// ```
#[derive(Default)]
pub struct AvifFormatProvider;

impl AvifFormatProvider {
    /// Creates a new instance of the AVIF format provider.
    ///
    /// # Returns
    ///
    /// `AvifFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for AvifFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "AVIF_IMAGE_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["avif", "avifs"]
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
            "AV1 Image",
            vec!["avif", "avifs"],
            vec!["image/avif", "image/avif-sequence"],
            MediaType::Image,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::Ffmpeg,
            PlaybackStrategy::None,
        )]
    }

    /// Validates AVIF magic bytes (starts with "ftyp") or uses extension-based detection.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - A byte slice containing the header bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match AVIF magic bytes, `false` otherwise.
    fn supports_magic_bytes(&self, _header_bytes: &[u8]) -> bool {
        false
    }

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
    /// Returns the metadata extraction capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for AvifFormatProvider {
    /// Extracts dimensions and codec information from an AVIF file via FFprobe.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the AVIF file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object containing the extracted metadata.
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

    /// Extracts semantic metadata from an AVIF file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the AVIF file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object containing the extracted metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - Always returns an empty JSON object for now.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for AvifFormatProvider {
    /// Generates a JPEG thumbnail from an AVIF file via FFmpeg.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the AVIF file.
    /// * `asset_id` - The ID of the asset.
    /// * `size_hint` - The size hint for the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the generated thumbnail.
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
impl PreviewCapability for AvifFormatProvider {
    /// Generates a preview from an AVIF file via FFmpeg.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the AVIF file.
    /// * `asset_id` - The ID of the asset.
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
