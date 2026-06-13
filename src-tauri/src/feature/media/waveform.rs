use crate::core::error::{AppError, AppResult};
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use std::path::Path;
use std::process::Command;
use tauri::AppHandle;

/// Extracts the audio waveform from a file as a normalized vector of floats.
///
/// Uses FFmpeg to stream raw 32-bit floats and downsamples them to a target number of points.
/// The FFmpeg subprocess is run on a blocking thread to avoid stalling the async runtime.
///
/// # Arguments
/// * `path` - Path to the audio/video file.
/// * `app_handle` - Tauri application handle to resolve FFmpeg location.
pub async fn extract_audio_waveform(path: &Path, app_handle: &AppHandle) -> AppResult<Vec<f32>> {
    let mut path_to_process = path.to_path_buf();
    let mut temp_dir_to_clean = None;

    // Fast path for MIDI: FFmpeg fails to extract waveform from sequence files.
    // We must synthesize it to a temporary WAV first.
    if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
        if ext == "mid" || ext == "midi" {
            let temp_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
            tokio::fs::create_dir_all(&temp_dir).await.map_err(|e| {
                AppError::Internal(format!("Failed to create temp dir for MIDI waveform: {}", e))
            })?;
            
            let temp_wav = temp_dir.join("waveform_temp.wav");
            crate::processing::media::extractors::midi_renderer::render_midi_to_wav(
                path,
                &temp_wav,
                Some(app_handle),
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to synthesize MIDI for waveform: {}", e)))?;
            
            path_to_process = temp_wav;
            temp_dir_to_clean = Some(temp_dir);
        }
    }

    let tools = resolve_transcoding_tools(Some(app_handle))?;
    let path_owned = path_to_process.clone();

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
    .map_err(|join_error| {
        AppError::Internal(format!("Waveform task panicked: {}", join_error))
    })??;

    if let Some(dir) = temp_dir_to_clean {
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    Ok(result)
}
