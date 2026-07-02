//! Audio extraction helpers shared by all audio format providers.
//!
//! Every audio provider in Mundam delegates to the same extraction pipeline:
//! FFprobe for technical metadata. This module centralises that logic so that
//! improvements and bug-fixes propagate to every audio format simultaneously.

use crate::core::error::{AppError, AppResult};
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Extracts technical metadata from an audio file using FFprobe.
///
/// Runs FFprobe in JSON mode, parses the output, and produces a simplified
/// metadata object suitable for the Mundam domain model (`AudioProbeResult`).
/// The output includes duration, codec, sample rate, channels, container format,
/// and a heuristic `is_native` flag indicating whether the codec can play in
/// a modern WebView without transcoding.
///
/// # Arguments
///
/// * `path` - Path to the audio file on disk.
///
/// # Returns
///
/// * `AppResult<Value>` - A JSON object with the extracted technical metadata.
///
/// # Errors
///
/// * `AppError::Transcoding` - If FFprobe fails to run or returns invalid JSON.
pub fn extract_audio_technical_metadata(path: &Path) -> AppResult<Value> {
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

    let mut duration_seconds = 0.0;
    let mut audio_codec = None;
    let mut sample_rate = None;
    let mut channels = None;
    let mut container = None;
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

    // Try to get the codec from the audio stream
    if let Some(streams_array) = probe_json
        .get("streams")
        .and_then(|streams| streams.as_array())
    {
        for stream_object in streams_array {
            if stream_object
                .get("codec_type")
                .and_then(|codec_type| codec_type.as_str())
                == Some("audio")
            {
                audio_codec = stream_object
                    .get("codec_name")
                    .and_then(|codec| codec.as_str())
                    .map(|string| string.to_string());
                sample_rate = stream_object
                    .get("sample_rate")
                    .and_then(|rate| rate.as_str())
                    .map(|string| string.to_string());
                channels = stream_object
                    .get("channels")
                    .and_then(|channels_val| channels_val.as_i64());
                break;
            }
        }
    }

    // Heuristic for is_native (V1 Parity)
    let is_native_audio = match audio_codec.as_deref() {
        Some("aac") | Some("mp3") | Some("mp2") | Some("flac") | Some("opus") | Some("vorbis") => {
            true
        }
        Some(codec) if codec.starts_with("pcm_") => true,
        _ => false,
    };

    technical_metadata.insert(
        "duration_secs".to_string(),
        serde_json::json!(duration_seconds),
    );
    technical_metadata.insert("audio_codec".to_string(), serde_json::json!(audio_codec));
    technical_metadata.insert("sample_rate".to_string(), serde_json::json!(sample_rate));
    technical_metadata.insert("channels".to_string(), serde_json::json!(channels));
    technical_metadata.insert("container".to_string(), serde_json::json!(container));
    technical_metadata.insert("bitrate_kbps".to_string(), serde_json::json!(bitrate_kbps));
    technical_metadata.insert("is_native".to_string(), serde_json::json!(is_native_audio));

    Ok(Value::Object(technical_metadata))
}
