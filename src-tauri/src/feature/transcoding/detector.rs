//! Transcoding Compatibility Detector
//!
//! Uses the central FileFormat registry to determine if a media file requires
//! transcoding for browser playback, and identifies the appropriate streaming strategy.

use crate::formats::{FileFormat, MediaType, PlaybackStrategy};
use std::path::Path;

/// Categorized media type for routing logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Video,
    Unknown,
}

/// Determines if a file path requires transcoding for browser playback.
///
/// # Arguments
/// * `path` - Path to the media file.
pub fn needs_transcoding(path: &Path) -> bool {
    if let Some(format) = FileFormat::detect(path) {
        !matches!(format.playback, PlaybackStrategy::Native)
    } else {
        // If we can't detect it, assume it's not native
        true
    }
}

/// Checks if a file is natively supported by the WebView (no transcoding needed).
pub fn is_native_format(path: &Path) -> bool {
    if let Some(format) = FileFormat::detect(path) {
        matches!(format.playback, PlaybackStrategy::Native)
    } else {
        false
    }
}

/// Determines the media kind (Audio/Video/Unknown) for a file.
pub fn get_media_kind(path: &Path) -> MediaKind {
    if let Some(format) = FileFormat::detect(path) {
        match format.type_category {
            MediaType::Audio => MediaKind::Audio,
            MediaType::Video => MediaKind::Video,
            _ => MediaKind::Unknown,
        }
    } else {
        MediaKind::Unknown
    }
}

/// Returns the playback strategy assigned to the file format.
pub fn get_playback_strategy(path: &Path) -> PlaybackStrategy {
    if let Some(format) = FileFormat::detect(path) {
        format.playback.clone()
    } else {
        PlaybackStrategy::None
    }
}

/// Returns the expected extension for a transcoded version of the file.
pub fn get_output_extension(path: &Path) -> &'static str {
    match get_media_kind(path) {
        MediaKind::Audio => "m4a",
        MediaKind::Video => "mp4",
        MediaKind::Unknown => "mp4",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detection_logic() {
        // These tests depend on definitions in SUPPORTED_FORMATS
        // Assuming .mp4 is native and .mkv is HLS
        assert!(!needs_transcoding(Path::new("video.mp4")));
        assert!(needs_transcoding(Path::new("video.mkv")));
        
        assert_eq!(get_media_kind(Path::new("audio.mp3")), MediaKind::Audio);
        assert_eq!(get_media_kind(Path::new("video.mp4")), MediaKind::Video);
    }
}
