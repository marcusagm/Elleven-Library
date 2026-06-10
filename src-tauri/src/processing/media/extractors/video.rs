//! Video extraction helpers shared by all video format providers.
//!
//! Every video provider in Mundam delegates to the same extraction pipeline:
//! FFprobe for technical metadata and FFmpeg for thumbnail frame-grabbing.
//! This module centralises that logic so that improvements and bug-fixes
//! propagate to every video format simultaneously.

use crate::core::error::{AppError, AppResult};
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Parses the frame-rate fraction returned by FFprobe (e.g. "30000/1001").
///
/// FFprobe returns `r_frame_rate` as a textual fraction. This function splits
/// numerator and denominator and returns the decimal value in fps, rounded to
/// two decimal places. Falls back to parsing the string as a plain number.
///
/// # Arguments
///
/// * `fraction_string` - The fraction string (e.g. "30000/1001", "25/1", "0/0").
///
/// # Returns
///
/// * `Option<f64>` - The frame rate in fps, or `None` if the fraction is invalid.
fn parse_frame_rate_fraction(fraction_string: &str) -> Option<f64> {
    let parts: Vec<&str> = fraction_string.split('/').collect();
    if parts.len() == 2 {
        let numerator = parts[0].parse::<f64>().ok()?;
        let denominator = parts[1].parse::<f64>().ok()?;
        if denominator > 0.0 {
            return Some((numerator / denominator * 100.0).round() / 100.0);
        }
    }
    // Fallback: try to parse as a direct number
    fraction_string.parse::<f64>().ok()
}

/// Extracts technical metadata from a video file using FFprobe.
///
/// Runs FFprobe in JSON mode, parses the output, and produces a simplified
/// metadata object suitable for the Mundam domain model (`VideoProbeResult`).
/// The output includes duration, codecs, dimensions, frame rate, bitrate, and
/// a heuristic `is_native` flag indicating whether the codec pair can play in
/// a modern WebView without transcoding.
///
/// # Arguments
///
/// * `path` - Path to the video file on disk.
///
/// # Returns
///
/// * `AppResult<Value>` - A JSON object with the extracted technical metadata.
///
/// # Errors
///
/// * `AppError::Transcoding` - If FFprobe fails to run or returns invalid JSON.
pub fn extract_video_technical_metadata(path: &Path) -> AppResult<Value> {
    let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

    let mut ffprobe_command = Command::new(tools.ffprobe);
    ffprobe_command.args([
        "-v",
        "error",
        "-show_format",
        "-show_streams",
        "-of",
        "json",
        &path.to_string_lossy(),
    ]);

    let ffprobe_output = run_command_with_timeout(ffprobe_command, 15)?;

    if !ffprobe_output.status.success() {
        let error_message = String::from_utf8_lossy(&ffprobe_output.stderr);
        return Err(AppError::Transcoding(format!(
            "FFprobe failed: {}",
            error_message
        )));
    }

    let probe_json: Value = serde_json::from_slice(&ffprobe_output.stdout).map_err(|error| {
        AppError::Transcoding(format!("Failed to parse FFprobe JSON: {}", error))
    })?;

    let mut technical_metadata = serde_json::Map::new();

    let mut video_codec = None;
    let mut audio_codec = None;
    let mut width = None;
    let mut height = None;
    let mut duration_seconds = 0.0;
    let mut container = None;
    let mut frame_rate_fps: Option<f64> = None;
    let mut bitrate_kbps: Option<f64> = None;

    if let Some(format_object) = probe_json.get("format") {
        if let Some(duration_string) = format_object
            .get("duration")
            .and_then(|duration| duration.as_str())
        {
            duration_seconds = duration_string.parse::<f64>().unwrap_or(0.0);
        }
        container = format_object
            .get("format_name")
            .and_then(|value| value.as_str())
            .map(|string| string.to_string());

        // FFprobe returns bit_rate as a string in bps; convert to kbps
        if let Some(bitrate_string) = format_object
            .get("bit_rate")
            .and_then(|bitrate| bitrate.as_str())
        {
            bitrate_kbps = bitrate_string
                .parse::<f64>()
                .ok()
                .map(|bits_per_second| (bits_per_second / 1000.0).round());
        }
    }

    if let Some(streams_array) = probe_json
        .get("streams")
        .and_then(|streams| streams.as_array())
    {
        for stream_object in streams_array {
            let codec_type = stream_object
                .get("codec_type")
                .and_then(|codec_type| codec_type.as_str());
            let codec_name = stream_object
                .get("codec_name")
                .and_then(|codec| codec.as_str())
                .map(|string| string.to_string());

            match codec_type {
                Some("video") if video_codec.is_none() => {
                    video_codec = codec_name;
                    width = stream_object
                        .get("width")
                        .and_then(|width_value| width_value.as_i64());
                    height = stream_object
                        .get("height")
                        .and_then(|height_value| height_value.as_i64());

                    // FFprobe returns r_frame_rate as a fraction (e.g. "30000/1001")
                    if let Some(frame_rate_string) = stream_object
                        .get("r_frame_rate")
                        .and_then(|value| value.as_str())
                    {
                        frame_rate_fps = parse_frame_rate_fraction(frame_rate_string);
                    }
                }
                Some("audio") if audio_codec.is_none() => {
                    audio_codec = codec_name;
                }
                _ => {}
            }
        }
    }

    // Heuristic for is_native (V1 Parity):
    // Check if video and audio codecs are natively supported in modern WebView (WebKit/Blink)
    let is_native_video = match video_codec.as_deref() {
        Some("h264") | Some("avc1") | Some("avc") | Some("vp8") => true,
        None => true, // Still or audio only
        _ => false,
    };

    let is_native_audio = match audio_codec.as_deref() {
        Some("aac") | Some("mp3") | Some("mp2") | Some("flac") | Some("opus")
        | Some("vorbis") => true,
        Some(codec) if codec.starts_with("pcm_") => true,
        None => true,
        _ => false,
    };

    let is_native_playback = is_native_video && is_native_audio;

    technical_metadata.insert(
        "duration_secs".to_string(),
        serde_json::json!(duration_seconds),
    );
    technical_metadata.insert("video_codec".to_string(), serde_json::json!(video_codec));
    technical_metadata.insert("audio_codec".to_string(), serde_json::json!(audio_codec));
    technical_metadata.insert("container".to_string(), serde_json::json!(container));
    technical_metadata.insert("width".to_string(), serde_json::json!(width));
    technical_metadata.insert("height".to_string(), serde_json::json!(height));
    technical_metadata.insert(
        "frame_rate_fps".to_string(),
        serde_json::json!(frame_rate_fps),
    );
    technical_metadata.insert("bitrate_kbps".to_string(), serde_json::json!(bitrate_kbps));
    technical_metadata.insert(
        "is_native".to_string(),
        serde_json::json!(is_native_playback),
    );

    Ok(Value::Object(technical_metadata))
}

/// Generates a JPEG thumbnail from a video file using FFmpeg frame extraction.
///
/// Attempts to capture a frame at the 1-second mark to avoid black intro frames.
/// If that fails (e.g. video shorter than 1s), retries from the very beginning
/// of the file. The frame is scaled using Lanczos filtering to match the
/// requested `size_hint` width.
///
/// # Arguments
///
/// * `path` - Path to the video file on disk.
/// * `size_hint` - The target width in pixels for the thumbnail.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The raw JPEG bytes of the extracted frame.
///
/// # Errors
///
/// * `AppError::Transcoding` - If FFmpeg fails to extract a frame after both attempts.
pub fn generate_video_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

    // First attempt: seek to 1 second to avoid black initial frames
    let mut first_attempt_command = Command::new(&tools.ffmpeg);
    first_attempt_command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-ss",
        "00:00:01",
        "-i",
        &path.to_string_lossy(),
        "-vf",
        &format!("scale={}:-1:flags=lanczos", size_hint),
        "-vframes",
        "1",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
        "-",
    ]);

    let first_attempt_output = run_command_with_timeout(first_attempt_command, 15)?;

    if first_attempt_output.status.success() {
        return Ok(first_attempt_output.stdout);
    }

    // Fallback: capture the very first frame (for very short videos)
    let mut fallback_command = Command::new(&tools.ffmpeg);
    fallback_command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        &path.to_string_lossy(),
        "-vf",
        &format!("scale={}:-1:flags=lanczos", size_hint),
        "-vframes",
        "1",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
        "-",
    ]);

    let fallback_output = run_command_with_timeout(fallback_command, 15)?;

    if fallback_output.status.success() {
        Ok(fallback_output.stdout)
    } else {
        let error_message = String::from_utf8_lossy(&fallback_output.stderr);
        Err(AppError::Transcoding(format!(
            "FFmpeg frame extraction failed: {}",
            error_message
        )))
    }
}
