use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MP3 Audio files (.mp3, .mp2).
///
/// Handles MPEG-1 and MPEG-2 Audio Layer III (and Layer II). These are
/// universally supported legacy audio formats that play natively in modern WebViews.
///
/// # Technical Details
///
/// - **Thumbnail Strategy**: Icon
/// - **Playback Strategy**: Native
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::audio::mp3::Mp3AudioProvider;
///
/// let provider = Mp3AudioProvider::new();
/// let formats = provider.supported_formats();
/// ```
#[derive(Default)]
pub struct Mp3AudioProvider;

impl Mp3AudioProvider {
    /// Creates a new instance of `Mp3AudioProvider`.
    ///
    /// # Returns
    ///
    /// `Mp3AudioProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for Mp3AudioProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "MP3_AUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mp3", "mp2"]
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
            "MP3 Audio",
            vec!["mp3", "mp2"],
            vec!["audio/mpeg"],
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
impl MetadataCapability for Mp3AudioProvider {
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
