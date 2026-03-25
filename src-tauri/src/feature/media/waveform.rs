use std::path::Path;
use std::process::Command;
use tauri::AppHandle;
use crate::core::error::{AppError, AppResult};
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};

/// Extracts the audio waveform from a file as a normalized vector of floats.
///
/// Uses FFmpeg to stream raw 32-bit floats and downsamples them to a target number of points.
/// The FFmpeg subprocess is run on a blocking thread to avoid stalling the async runtime.
///
/// # Arguments
/// * `path` - Path to the audio/video file.
/// * `app_handle` - Tauri application handle to resolve FFmpeg location.
pub async fn extract_audio_waveform(path: &Path, app_handle: &AppHandle) -> AppResult<Vec<f32>> {
    let tools = resolve_transcoding_tools(Some(app_handle))?;
    let path_owned = path.to_path_buf();

    // Run the blocking FFmpeg subprocess on a dedicated thread
    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = Command::new(tools.ffmpeg);
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &path_owned.to_string_lossy(),
            "-ar",
            "100", // Low sample rate for analysis
            "-ac",
            "1", // Mono
            "-f",
            "f32le", // 32-bit float little endian
            "-",     // Output to stdout
        ]);

        let output = run_command_with_timeout(cmd, 30)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Transcoding(format!(
                "FFmpeg waveform extraction failed for {:?}: {}",
                path_owned, error
            )));
        }

        let raw_data = output.stdout;
        let floats: Vec<f32> = raw_data
            .chunks_exact(4)
            .map(|chunk| {
                let byte_array = [chunk[0], chunk[1], chunk[2], chunk[3]];
                f32::from_le_bytes(byte_array).abs()
            })
            .collect();

        if floats.is_empty() {
            return Ok(vec![]);
        }

        // Downsample to 500 points for the UI
        let target_points = 500;
        let result = if floats.len() <= target_points {
            floats
        } else {
            let chunk_size = floats.len() / target_points;
            floats
                .chunks(chunk_size)
                .map(|chunk| chunk.iter().fold(0.0f32, |max, &val| max.max(val)))
                .take(target_points)
                .collect()
        };

        // Normalize to [0.0, 1.0]
        let max_amplitude = result.iter().fold(0.0f32, |max, &val| max.max(val));
        if max_amplitude > 0.0 {
            Ok(result.iter().map(|&value| value / max_amplitude).collect())
        } else {
            Ok(result)
        }
    })
    .await
    .map_err(|join_error| AppError::Internal(format!("Waveform task panicked: {}", join_error)))??;

    Ok(result)
}

