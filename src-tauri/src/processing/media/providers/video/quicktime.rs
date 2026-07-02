use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Apple QuickTime video files (.mov, .qt).
///
/// QuickTime is Apple's native multimedia container format. Both `.mov` and
/// `.qt` are aliases for the same QuickTime File Format. Modern QuickTime files
/// using H.264/AAC typically play natively in WebViews, but older codec
/// combinations may require HLS transcoding.
///
/// # Technical Details
///
/// - **Container**: QuickTime File Format
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Preview Strategy**: Browser-native playback
/// - **Playback Strategy**: Native (no HLS required for common codecs)
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::video::quicktime::QuicktimeVideoProvider;
///
/// let provider = QuicktimeVideoProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats[0].name, "QuickTime Video");
/// assert_eq!(formats[0].extensions, vec!["mov", "qt"]);
/// ```
#[derive(Default)]
pub struct QuicktimeVideoProvider;

impl QuicktimeVideoProvider {
    /// Creates a new instance of `QuicktimeVideoProvider`.
    ///
    /// # Returns
    ///
    /// `QuicktimeVideoProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for QuicktimeVideoProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "QUICKTIME_VIDEO_PROVIDER"
    }

    /// Returns the file extensions handled by this provider.
    ///
    /// `.qt` is an alias for the QuickTime format.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mov", "qt"]
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
            "QuickTime Video",
            vec!["mov", "qt"],
            vec!["video/quicktime"],
            MediaType::Video,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::Native,
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
impl MetadataCapability for QuicktimeVideoProvider {
    /// Extracts technical metadata (codecs, dimensions, duration) via FFprobe.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the QuickTime video file.
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
impl ThumbnailCapability for QuicktimeVideoProvider {
    /// Generates a JPEG thumbnail via FFmpeg frame extraction.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the QuickTime video file.
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
