use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Material Exchange Format video files (.mxf).
///
/// MXF is a professional broadcast container format standardised by SMPTE.
/// It is widely used in professional video production and post-production
/// workflows. Browsers cannot play MXF natively, so playback requires HLS
/// transcoding via the streaming server.
///
/// # Technical Details
///
/// - **Container**: Material Exchange Format (SMPTE 377M)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Preview Strategy**: FFmpeg transcoding
/// - **Playback Strategy**: HLS (requires streaming server)
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::video::mxf::MxfVideoProvider;
///
/// let provider = MxfVideoProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats[0].name, "Material Exchange Format");
/// assert_eq!(formats[0].extensions, vec!["mxf"]);
/// ```
#[derive(Default)]
pub struct MxfVideoProvider;

impl MxfVideoProvider {
    /// Creates a new instance of `MxfVideoProvider`.
    ///
    /// # Returns
    ///
    /// `MxfVideoProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for MxfVideoProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "MXF_VIDEO_PROVIDER"
    }

    /// Returns the file extensions handled by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mxf"]
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
            "Material Exchange Format",
            vec!["mxf"],
            vec!["video/mxf"],
            MediaType::Video,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::Ffmpeg,
            PlaybackStrategy::Hls,
        )]
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
}

#[async_trait]
impl MetadataCapability for MxfVideoProvider {
    /// Extracts technical metadata (codecs, dimensions, duration) via FFprobe.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MXF video file.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFprobe fails or returns invalid output.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::video::extract_video_technical_metadata(
                &path_owned,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Returns empty semantic metadata (no semantic extraction for video).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the video file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for MxfVideoProvider {
    /// Generates a JPEG thumbnail via FFmpeg frame extraction.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MXF video file.
    /// * `_asset_id` - Unique identifier for the asset (unused).
    /// * `size_hint` - Requested maximum width in pixels.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFmpeg fails to extract a frame.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::video::generate_video_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
