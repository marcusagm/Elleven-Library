use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MPEG-4 Audio files (.m4a, .m4r, .aac, .m4b).
///
/// Handles audio stored in the MPEG-4 container (or raw AAC). Very common
/// format due to iTunes/Apple ecosystem. Supported natively by WebViews.
///
/// # Technical Details
///
/// - **Thumbnail Strategy**: Icon
/// - **Playback Strategy**: Native
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::audio::mpeg4_audio::Mpeg4AudioProvider;
///
/// let provider = Mpeg4AudioProvider::new();
/// let formats = provider.supported_formats();
/// ```
#[derive(Default)]
pub struct Mpeg4AudioProvider;

impl Mpeg4AudioProvider {
    /// Creates a new instance of `Mpeg4AudioProvider`.
    ///
    /// # Returns
    ///
    /// `Mpeg4AudioProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for Mpeg4AudioProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "MPEG4_AUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["m4a", "m4r", "aac", "m4b"]
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
            "MPEG-4 Audio",
            vec!["m4a", "m4r", "aac", "m4b"],
            vec!["audio/mp4", "audio/aac"],
            MediaType::Audio,
            ThumbnailStrategy::Icon,
            PreviewStrategy::None,
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
}

#[async_trait]
impl MetadataCapability for Mpeg4AudioProvider {
    /// Extracts technical metadata such as codec, sample rate, and channels.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Technical metadata JSON object.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If FFprobe fails to run or returns invalid JSON.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::audio::extract_audio_technical_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata (currently empty for audio).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the audio file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Semantic metadata JSON object.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}
