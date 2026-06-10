//! Per-format video providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct video container or codec family and delegates
//! all extraction logic to the shared functions in
//! `crate::processing::media::extractors::video`. This keeps the provider
//! layer thin and ensures that bug-fixes to the core FFmpeg/FFprobe pipeline
//! benefit every video format simultaneously.
//!
//! # Organisation
//!
//! | Category              | Modules                                          |
//! |-----------------------|--------------------------------------------------|
//! | Native playback       | `mpeg4`, `webm`, `quicktime`                     |
//! | HLS transcoding       | `matroska`, `mxf`, `windows_media`, `flash_video`|
//! |                       | `mpeg_transport_stream`, `avi`, `three_gpp`      |
//! |                       | `realmedia`, `windows_recorded_tv`, `ogg_video`  |
//! | Linear HLS            | `shockwave_flash`, `mpeg_video`, `motion_jpeg`   |
//! |                       | `hevc`, `h264_raw`, `yuv4mpeg2`                  |

pub mod avi;
pub mod flash_video;
pub mod h264_raw;
pub mod hevc;
pub mod matroska;
pub mod motion_jpeg;
pub mod mpeg4;
pub mod mpeg_transport_stream;
pub mod mpeg_video;
pub mod mxf;
pub mod ogg_video;
pub mod quicktime;
pub mod realmedia;
pub mod shockwave_flash;
pub mod three_gpp;
pub mod webm;
pub mod windows_media;
pub mod windows_recorded_tv;
pub mod yuv4mpeg2;
