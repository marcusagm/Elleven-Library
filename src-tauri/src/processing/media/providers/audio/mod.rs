//! Per-format audio providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct audio container or codec family and delegates
//! all extraction logic to the shared functions in
//! `crate::processing::media::extractors::audio`. This keeps the provider
//! layer thin and ensures that bug-fixes to the core FFmpeg/FFprobe pipeline
//! benefit every audio format simultaneously.
//!
//! # Organisation
//!
//! | Category              | Modules                                          |
//! |-----------------------|--------------------------------------------------|
//! | Native playback       | `mp3`, `wav`, `flac`, `mpeg4_audio`              |
//! | Audio HLS transcoding | `ogg_audio`, `aiff`, `windows_media_audio`       |
//! |                       | `opus`, `midi`, `matroska_audio`, `speex`        |
//! |                       | `monkeys_audio`, `wavpack`, `dolby_digital`      |
//! |                       | `dts`, `amr`, `apple_core_audio`, `audible`      |
//! |                       | `realaudio`, `musepack`                          |

pub mod aiff;
pub mod amr;
pub mod apple_core_audio;
pub mod audible;
pub mod dolby_digital;
pub mod dts;
pub mod flac;
pub mod matroska_audio;
pub mod midi;
pub mod monkeys_audio;
pub mod mp3;
pub mod mpeg4_audio;
pub mod musepack;
pub mod ogg_audio;
pub mod opus;
pub mod realaudio;
pub mod speex;
pub mod wav;
pub mod wavpack;
pub mod windows_media_audio;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all audio format providers into a single vector.
///
/// This function is the single point of registration for all audio providers.
/// New audio formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All audio format providers, ordered with native-playback formats first,
/// followed by formats requiring HLS transcoding.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(mp3::Mp3AudioProvider::new()),
        Arc::new(wav::WavAudioProvider::new()),
        Arc::new(flac::FlacAudioProvider::new()),
        Arc::new(ogg_audio::OggAudioProvider::new()),
        Arc::new(mpeg4_audio::Mpeg4AudioProvider::new()),
        Arc::new(aiff::AiffAudioProvider::new()),
        Arc::new(windows_media_audio::WindowsMediaAudioProvider::new()),
        Arc::new(opus::OpusAudioProvider::new()),
        Arc::new(midi::MidiAudioProvider::new()),
        Arc::new(matroska_audio::MatroskaAudioProvider::new()),
        Arc::new(speex::SpeexAudioProvider::new()),
        Arc::new(monkeys_audio::MonkeysAudioProvider::new()),
        Arc::new(wavpack::WavpackAudioProvider::new()),
        Arc::new(dolby_digital::DolbyDigitalAudioProvider::new()),
        Arc::new(dts::DtsAudioProvider::new()),
        Arc::new(amr::AmrAudioProvider::new()),
        Arc::new(apple_core_audio::AppleCoreAudioProvider::new()),
        Arc::new(audible::AudibleAudioProvider::new()),
        Arc::new(realaudio::RealaudioProvider::new()),
        Arc::new(musepack::MusepackAudioProvider::new()),
    ]
}
