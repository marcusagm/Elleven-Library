use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for HEVC/H.265 raw video files (.hevc, .h265).
///
/// Handles raw High Efficiency Video Coding bitstreams. These are elementary
/// streams without a container, used primarily for testing and professional
/// workflows. Browser support for HEVC varies significantly, so Linear HLS
/// transcoding is used for reliable playback.
///
/// # Technical Details
///
/// - **Format**: HEVC / H.265 Elementary Stream
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: Linear HLS (sequential transcoding)
///
/// # Known Issues
///
/// Some HEVC streams may cause instability in the M3U8 local decoder
/// (tracked since Sprint 10.12). Thumbnail extraction remains stable.
#[derive(Default)]
pub struct HevcVideoProvider;

impl HevcVideoProvider {
    /// Creates a new instance of `HevcVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for HevcVideoProvider {
    fn name(&self) -> &'static str {
        "HEVC_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["hevc", "h265"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "HEVC Video",
            vec!["hevc", "h265"],
            vec!["video/hevc"],
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
impl MetadataCapability for HevcVideoProvider {
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
impl ThumbnailCapability for HevcVideoProvider {
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
