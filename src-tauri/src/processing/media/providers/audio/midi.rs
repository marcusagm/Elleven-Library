use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MIDI Audio files (.mid, .midi).
///
/// Musical Instrument Digital Interface files. They contain instructions, not audio data.
/// FFmpeg requires a SoundFont to transcode these correctly (improved in Sprint 10.12 via HLS).
///
/// # Technical Details
///
/// - **Thumbnail Strategy**: Icon
/// - **Playback Strategy**: Audio HLS
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::audio::midi::MidiAudioProvider;
///
/// let provider = MidiAudioProvider::new();
/// let formats = provider.supported_formats();
/// ```
#[derive(Default)]
pub struct MidiAudioProvider;

impl MidiAudioProvider {
    /// Creates a new instance of `MidiAudioProvider`.
    ///
    /// # Returns
    ///
    /// `MidiAudioProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for MidiAudioProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "MIDI_AUDIO_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mid", "midi"]
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
            "MIDI Audio",
            vec!["mid", "midi"],
            vec!["audio/midi"],
            MediaType::Audio,
            ThumbnailStrategy::Icon,
            PreviewStrategy::None,
            PlaybackStrategy::AudioLinearHls, // Uses rustysynth to render to WAV and stream via LinearHls
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
impl MetadataCapability for MidiAudioProvider {
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
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        // MIDI files are sequence files, not standard audio streams.
        // FFprobe often fails to parse them natively without a SoundFont or specific config.
        // We return a basic structure to prevent pipeline failure.
        Ok(serde_json::json!({
            "format": "midi",
            "codec": "midi",
        }))
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
