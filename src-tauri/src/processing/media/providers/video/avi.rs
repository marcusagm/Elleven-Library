use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for AVI video files (.avi, .divx).
///
/// Handles Microsoft's legacy Audio Video Interleave container. DivX is
/// treated as an alias since `.divx` files use the AVI container with
/// DivX-encoded video. Requires HLS transcoding for browser playback.
///
/// # Technical Details
///
/// - **Container**: Audio Video Interleave (AVI)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: HLS (requires streaming server)
#[derive(Default)]
pub struct AviVideoProvider;

impl AviVideoProvider {
    /// Creates a new instance of `AviVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for AviVideoProvider {
    fn name(&self) -> &'static str {
        "AVI_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["avi", "divx"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "AVI Video",
            vec!["avi", "divx"],
            vec!["video/x-msvideo"],
            MediaType::Video,
            ThumbnailStrategy::Ffmpeg,
            PreviewStrategy::Ffmpeg,
            PlaybackStrategy::Hls,
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
impl MetadataCapability for AviVideoProvider {
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
impl ThumbnailCapability for AviVideoProvider {
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
