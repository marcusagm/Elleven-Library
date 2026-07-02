use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MPEG-4 video files (.mp4, .m4v).
///
/// Handles the most common video container format. Both `.mp4` and `.m4v`
/// (Apple's variant) share the same ISO Base Media File Format container.
/// These formats typically play natively in modern WebViews without
/// transcoding when using H.264/AAC codecs.
///
/// # Technical Details
///
/// - **Container**: ISO Base Media File Format (MPEG-4 Part 14)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Preview Strategy**: Browser-native playback
/// - **Playback Strategy**: Native (no HLS required)
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::video::mpeg4::Mpeg4VideoProvider;
///
/// let provider = Mpeg4VideoProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats[0].name, "MPEG-4 Video");
/// assert_eq!(formats[0].extensions, vec!["mp4", "m4v"]);
/// ```
#[derive(Default)]
pub struct Mpeg4VideoProvider;

impl Mpeg4VideoProvider {
    /// Creates a new instance of `Mpeg4VideoProvider`.
    ///
    /// # Returns
    ///
    /// `Mpeg4VideoProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for Mpeg4VideoProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "MPEG4_VIDEO_PROVIDER"
    }

    /// Returns the file extensions handled by this provider.
    ///
    /// `.m4v` is Apple's MPEG-4 variant and is treated identically.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mp4", "m4v"]
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
            "MPEG-4 Video",
            vec!["mp4", "m4v"],
            vec!["video/mp4"],
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
impl MetadataCapability for Mpeg4VideoProvider {
    /// Extracts technical metadata (codecs, dimensions, duration) via FFprobe.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MPEG-4 video file.
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
impl ThumbnailCapability for Mpeg4VideoProvider {
    /// Generates a JPEG thumbnail via FFmpeg frame extraction.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MPEG-4 video file.
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
