use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Dolby Digital Audio files (.ac3).
///
/// Dolby Digital is an audio compression technology developed by Dolby Laboratories.
/// Browsers do not support it natively, so we transcode via Audio HLS.
///
/// # Technical Details
///
/// - **Thumbnail Strategy**: Icon
/// - **Playback Strategy**: Audio HLS
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::audio::dolby_digital::DolbyDigitalAudioProvider;
///
/// let provider = DolbyDigitalAudioProvider::new();
/// let formats = provider.supported_formats();
/// ```
#[derive(Default)]
pub struct DolbyDigitalAudioProvider;

impl DolbyDigitalAudioProvider {
    /// Creates a new instance of `DolbyDigitalAudioProvider`.
    ///
    /// # Returns
    ///
    /// `DolbyDigitalAudioProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for DolbyDigitalAudioProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "DOLBY_DIGITAL_AUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ac3"]
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
            "Dolby Digital",
            vec!["ac3"],
            vec!["audio/ac3"],
            MediaType::Audio,
            ThumbnailStrategy::Icon,
            PreviewStrategy::None,
            PlaybackStrategy::AudioHls,
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
impl MetadataCapability for DolbyDigitalAudioProvider {
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
