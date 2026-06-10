use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for YUV4MPEG2 video files (.y4m).
///
/// YUV4MPEG2 is a simple uncompressed video format used primarily in
/// video processing toolchains (e.g. x264/x265 encoding pipelines) and
/// testing. Files can be extremely large since every frame is stored as
/// raw YUV data. Requires Linear HLS for sequential transcoding.
///
/// # Technical Details
///
/// - **Format**: YUV4MPEG2 (uncompressed YUV frames with text header)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: Linear HLS (sequential transcoding)
#[derive(Default)]
pub struct Yuv4mpeg2VideoProvider;

impl Yuv4mpeg2VideoProvider {
    /// Creates a new instance of `Yuv4mpeg2VideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for Yuv4mpeg2VideoProvider {
    fn name(&self) -> &'static str {
        "YUV4MPEG2_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["y4m"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "YUV4MPEG2 Video",
            vec!["y4m"],
            vec!["video/x-y4m"],
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
impl MetadataCapability for Yuv4mpeg2VideoProvider {
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
impl ThumbnailCapability for Yuv4mpeg2VideoProvider {
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
