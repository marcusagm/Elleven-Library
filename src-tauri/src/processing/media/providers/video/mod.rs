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

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all video format providers into a single vector.
///
/// This function is the single point of registration for all video providers.
/// New video formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All video format providers, ordered with native-playback formats first,
/// followed by HLS transcoding, then linear HLS formats.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(mpeg4::Mpeg4VideoProvider::new()),
        Arc::new(webm::WebmVideoProvider::new()),
        Arc::new(quicktime::QuicktimeVideoProvider::new()),
        Arc::new(matroska::MatroskaVideoProvider::new()),
        Arc::new(mxf::MxfVideoProvider::new()),
        Arc::new(windows_media::WindowsMediaVideoProvider::new()),
        Arc::new(flash_video::FlashVideoProvider::new()),
        Arc::new(shockwave_flash::ShockwaveFlashProvider::new()),
        Arc::new(mpeg_video::MpegVideoProvider::new()),
        Arc::new(mpeg_transport_stream::MpegTransportStreamProvider::new()),
        Arc::new(avi::AviVideoProvider::new()),
        Arc::new(three_gpp::ThreeGppVideoProvider::new()),
        Arc::new(realmedia::RealmediaVideoProvider::new()),
        Arc::new(windows_recorded_tv::WindowsRecordedTvProvider::new()),
        Arc::new(ogg_video::OggVideoProvider::new()),
        Arc::new(motion_jpeg::MotionJpegVideoProvider::new()),
        Arc::new(hevc::HevcVideoProvider::new()),
        Arc::new(h264_raw::H264RawVideoProvider::new()),
        Arc::new(yuv4mpeg2::Yuv4mpeg2VideoProvider::new()),
    ]
}
