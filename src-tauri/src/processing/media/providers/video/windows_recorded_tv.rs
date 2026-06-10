use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Windows Recorded TV Show files (.wtv).
///
/// WTV is Microsoft's container for Windows Media Center recordings.
/// It can contain MPEG-2 or H.264 video with AC-3 or AAC audio.
/// Requires HLS transcoding for browser playback.
///
/// # Technical Details
///
/// - **Container**: Windows Television (WTV)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: HLS (requires streaming server)
#[derive(Default)]
pub struct WindowsRecordedTvProvider;

impl WindowsRecordedTvProvider {
    /// Creates a new instance of `WindowsRecordedTvProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for WindowsRecordedTvProvider {
    fn name(&self) -> &'static str {
        "WINDOWS_RECORDED_TV_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wtv"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "Windows Recorded TV",
            vec!["wtv"],
            vec!["video/x-wtv"],
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
impl MetadataCapability for WindowsRecordedTvProvider {
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
impl ThumbnailCapability for WindowsRecordedTvProvider {
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
