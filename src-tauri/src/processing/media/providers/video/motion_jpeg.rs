use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Motion JPEG video files (.mjpeg, .mjpg).
///
/// Motion JPEG encodes each frame as an independent JPEG image. It is
/// commonly used by webcams, IP cameras, and surveillance systems.
/// Requires Linear HLS for sequential transcoding since browsers cannot
/// play raw MJPEG streams.
///
/// # Technical Details
///
/// - **Format**: Motion JPEG (frame-by-frame JPEG encoding)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: Linear HLS (sequential transcoding)
#[derive(Default)]
pub struct MotionJpegVideoProvider;

impl MotionJpegVideoProvider {
    /// Creates a new instance of `MotionJpegVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for MotionJpegVideoProvider {
    fn name(&self) -> &'static str {
        "MOTION_JPEG_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mjpeg", "mjpg"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "Motion JPEG",
            vec!["mjpeg", "mjpg"],
            vec!["video/x-motion-jpeg"],
            MediaType::Video,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::Ffmpeg,
            PlaybackStrategy::LinearHls,
        )]
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for MotionJpegVideoProvider {
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::video::extract_video_technical_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for MotionJpegVideoProvider {
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::video::generate_video_thumbnail(&path_owned, size_hint)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
