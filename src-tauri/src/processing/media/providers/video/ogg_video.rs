use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Ogg Video files (.ogv).
///
/// OGV is the Xiph.Org Foundation's open video container based on Ogg.
/// It typically wraps Theora video and Vorbis audio streams. Despite being
/// an open format, browser support is inconsistent, so HLS transcoding
/// is used for reliable playback.
///
/// # Technical Details
///
/// - **Container**: Ogg
/// - **Thumbnail Strategy**: FFmpeg frame extraction
/// - **Playback Strategy**: HLS (requires streaming server)
#[derive(Default)]
pub struct OggVideoProvider;

impl OggVideoProvider {
    /// Creates a new instance of `OggVideoProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for OggVideoProvider {
    fn name(&self) -> &'static str {
        "OGG_VIDEO_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ogv"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };
        vec![SupportedFormat::with_metadata(
            "Ogg Video",
            vec!["ogv"],
            vec!["video/ogg"],
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
impl MetadataCapability for OggVideoProvider {
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
impl ThumbnailCapability for OggVideoProvider {
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
