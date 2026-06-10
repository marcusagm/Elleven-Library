use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for 3GPP video files (.3gp, .3g2).
///
/// Handles the 3rd Generation Partnership Project multimedia containers.
/// `.3gp` is the original 3GPP format; `.3g2` is the 3GPP2 variant used
/// by CDMA networks. Both are MPEG-4 Part 12 derivatives optimised for
/// mobile devices. Requires HLS transcoding for browser playback.
///
/// # Technical Details
///
/// - **Container**: 3GPP / 3GPP2 (ISO Base Media File Format derivative)
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: HLS (requires streaming server)
#[derive(Default)]
pub struct ThreeGppVideoProvider;

impl ThreeGppVideoProvider {
    /// Creates a new instance of `ThreeGppVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for ThreeGppVideoProvider {
    fn name(&self) -> &'static str {
        "THREE_GPP_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["3gp", "3g2"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "3GPP Video",
            vec!["3gp", "3g2"],
            vec!["video/3gpp"],
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
impl MetadataCapability for ThreeGppVideoProvider {
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
impl ThumbnailCapability for ThreeGppVideoProvider {
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
