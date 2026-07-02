use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Waveform Audio files (.wav, .bwf).
///
/// Handles uncompressed PCM audio formats developed by Microsoft and IBM,
/// as well as Broadcast Wave Format variants used in professional environments.
/// Playback is native in modern browsers.
///
/// # Technical Details
///
/// - **Thumbnail Strategy**: Icon
/// - **Playback Strategy**: Native
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::audio::wav::WavAudioProvider;
///
/// let provider = WavAudioProvider::new();
/// let formats = provider.supported_formats();
/// ```
#[derive(Default)]
pub struct WavAudioProvider;

impl WavAudioProvider {
    /// Creates a new instance of `WavAudioProvider`.
    ///
    /// # Returns
    ///
    /// `WavAudioProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for WavAudioProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "WAV_AUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["wav", "bwf"]
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
            "Waveform Audio",
            vec!["wav", "bwf"],
            vec!["audio/wav", "audio/x-wav"],
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
impl MetadataCapability for WavAudioProvider {
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
            crate::processing::media::extractors::audio::extract_audio_technical_metadata(
                &path_owned,
            )
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
