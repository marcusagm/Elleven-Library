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
