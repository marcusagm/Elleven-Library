use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for H.264 raw video files (.h264).
///
/// Handles raw H.264/AVC elementary bitstreams without a container. These
/// files are typically produced by video encoders for testing or muxing into
/// MP4/MKV. Requires Linear HLS for sequential transcoding since browsers
/// need a proper container for playback.
///
/// # Technical Details
///
/// - **Format**: H.264/AVC Elementary Stream
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: Linear HLS (sequential transcoding)
#[derive(Default)]
pub struct H264RawVideoProvider;

impl H264RawVideoProvider {
    /// Creates a new instance of `H264RawVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for H264RawVideoProvider {
    fn name(&self) -> &'static str {
        "H264_RAW_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["h264"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "H.264 Raw Video",
            vec!["h264"],
            vec!["video/h264"],
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
impl MetadataCapability for H264RawVideoProvider {
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
impl ThumbnailCapability for H264RawVideoProvider {
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
