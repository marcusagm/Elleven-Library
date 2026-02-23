//! M3U8 Playlist Generator
//!
//! Generates HLS playlists dynamically based on video duration.

/// Generate an M3U8 playlist for a video file (test-only; production uses [`generate_m3u8_with_token`]).
///
/// # Arguments
/// * `file_path` - URL-encoded path to the video file (used in segment URLs)
/// * `duration_secs` - Total duration of the video in seconds
/// * `segment_duration` - Duration of each segment in seconds
/// * `quality` - Transcoding quality (preview, standard, high)
#[cfg(test)]
pub fn generate_m3u8(
    file_path: &str,
    duration_secs: f64,
    segment_duration: f64,
    quality: &str,
) -> String {
    let num_segments = (duration_secs / segment_duration).ceil() as u32;

    let mut playlist = String::new();

    // Header
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:3\n");
    playlist.push_str(&format!(
        "#EXT-X-TARGETDURATION:{}\n",
        segment_duration.ceil() as u32
    ));
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");

    // Segments
    for i in 0..num_segments {
        let seg_start = i as f64 * segment_duration;
        let seg_duration = (duration_secs - seg_start).min(segment_duration);

        playlist.push_str(&format!("#EXTINF:{:.3},\n", seg_duration));
        playlist.push_str(&format!(
            "/segment/{}/{}.ts?quality={}\n",
            file_path, i, quality
        ));
    }

    // End marker
    playlist.push_str("#EXT-X-ENDLIST\n");

    playlist
}

/// Generate an M3U8 playlist with session token embedded in segment URLs.
///
/// The `token_suffix` is appended to each segment URL so that HLS.js
/// can authenticate every segment request without custom `xhrSetup`.
///
/// # Arguments
/// * `file_path` - URL-encoded path to the video file (used in segment URLs)
/// * `duration_secs` - Total duration of the video in seconds
/// * `segment_duration` - Duration of each segment in seconds
/// * `quality` - Transcoding quality (preview, standard, high)
/// * `token_suffix` - Token query param to append (e.g., "&token=abc123"), or empty string
pub fn generate_m3u8_with_token(
    file_path: &str,
    duration_secs: f64,
    segment_duration: f64,
    quality: &str,
    token_suffix: &str,
) -> String {
    let num_segments = (duration_secs / segment_duration).ceil() as u32;

    let mut playlist = String::new();

    // Header
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:3\n");
    playlist.push_str(&format!(
        "#EXT-X-TARGETDURATION:{}\n",
        segment_duration.ceil() as u32
    ));
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");

    // Segments — each URL includes the token so HLS.js can fetch them authenticated
    for segment_index in 0..num_segments {
        let segment_start = segment_index as f64 * segment_duration;
        let actual_segment_duration = (duration_secs - segment_start).min(segment_duration);

        playlist.push_str(&format!("#EXTINF:{:.3},\n", actual_segment_duration));
        playlist.push_str(&format!(
            "/segment/{}/{}.ts?quality={}{}\n",
            file_path, segment_index, quality, token_suffix
        ));
    }

    // End marker
    playlist.push_str("#EXT-X-ENDLIST\n");

    playlist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_m3u8_short_video() {
        let m3u8 = generate_m3u8("test.mkv", 25.0, 10.0, "standard");

        assert!(m3u8.contains("#EXTM3U"));
        assert!(m3u8.contains("#EXT-X-TARGETDURATION:10"));
        assert!(m3u8.contains("/segment/test.mkv/0.ts?quality=standard"));
        assert!(m3u8.contains("/segment/test.mkv/1.ts?quality=standard"));
        assert!(m3u8.contains("/segment/test.mkv/2.ts?quality=standard"));
        assert!(m3u8.contains("#EXT-X-ENDLIST"));
    }

    #[test]
    fn test_generate_m3u8_exact_duration() {
        let m3u8 = generate_m3u8("video.mkv", 30.0, 10.0, "preview");

        // Should have exactly 3 segments for 30 seconds
        assert!(m3u8.contains("/segment/video.mkv/0.ts?quality=preview"));
        assert!(m3u8.contains("/segment/video.mkv/1.ts?quality=preview"));
        assert!(m3u8.contains("/segment/video.mkv/2.ts?quality=preview"));
        assert!(!m3u8.contains("/segment/video.mkv/3.ts"));
    }

    #[test]
    fn test_generate_m3u8_long_video() {
        let m3u8 = generate_m3u8("long.mkv", 3600.0, 10.0, "high");

        // 1 hour = 360 segments
        assert!(m3u8.contains("/segment/long.mkv/359.ts?quality=high"));
        assert!(!m3u8.contains("/segment/long.mkv/360.ts"));
    }

    #[test]
    fn test_generate_m3u8_with_token() {
        let m3u8 = generate_m3u8_with_token("test.mkv", 25.0, 10.0, "standard", "&token=abc123");

        assert!(m3u8.contains("/segment/test.mkv/0.ts?quality=standard&token=abc123"));
        assert!(m3u8.contains("/segment/test.mkv/1.ts?quality=standard&token=abc123"));
        assert!(m3u8.contains("/segment/test.mkv/2.ts?quality=standard&token=abc123"));
    }

    #[test]
    fn test_generate_m3u8_with_empty_token() {
        let m3u8 = generate_m3u8_with_token("test.mkv", 25.0, 10.0, "standard", "");

        // Without token, URLs should be identical to the original format
        assert!(m3u8.contains("/segment/test.mkv/0.ts?quality=standard\n"));
    }
}
