//! M3U8 Playlist Generator
//!
//! Generates HLS playlists dynamically based on video duration.

/// Generate an M3U8 playlist with session token embedded in segment URLs.
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
