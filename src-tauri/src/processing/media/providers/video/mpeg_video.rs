use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MPEG-1/2 video files (.mpg, .mpeg, .m2v).
///
/// Handles the legacy MPEG-1 and MPEG-2 elementary stream formats. These are
/// raw video bitstreams without a modern container, requiring Linear HLS
/// transcoding for sequential playback in the browser.
///
/// # Technical Details
///
/// - **Format**: MPEG-1/2 Video Elementary Stream
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: Linear HLS (sequential transcoding)
#[derive(Default)]
pub struct MpegVideoProvider;

impl MpegVideoProvider {
    /// Creates a new instance of `MpegVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for MpegVideoProvider {
    fn name(&self) -> &'static str {
        "MPEG_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mpg", "mpeg", "m2v"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "MPEG-1/2 Video",
            vec!["mpg", "mpeg", "m2v"],
            vec!["video/mpeg"],
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
impl MetadataCapability for MpegVideoProvider {
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

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for MpegVideoProvider {
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
