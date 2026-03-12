//! Provedor de formato de áudio.
//!
//! Utiliza FFprobe para metadados e FFmpeg em modo pipe para extração
//! de waveforms (amplitude de áudio).

use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Provedor de formato de áudio.
pub struct AudioFormatProvider {}

/// Implementação do provedor de formato de áudio.
impl AudioFormatProvider {
    /// Cria uma nova instância do provedor.
    ///
    /// # Returns
    ///
    /// * `Self` - A nova instância do provedor.
    pub fn new() -> Self {
        Self {}
    }

    /// Extrai a waveform do áudio como um vetor de floats normalizados (0.0 a 1.0).
    ///
    /// Utiliza o pipeline: ffmpeg -> f32le stream -> buffer -> normalização.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo de áudio.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<f32>>` - O resultado da extração da waveform.
    pub async fn get_waveform(&self, path: &Path) -> AppResult<Vec<f32>> {
        let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

        let mut cmd = Command::new(tools.ffmpeg);
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            &path.to_string_lossy(),
            "-ar",
            "100", // Sample rate baixo para reduzir dados (100 samples por segundo de áudio)
            "-ac",
            "1", // Mono
            "-f",
            "f32le", // 32-bit float little endian
            "-",     // Output para stdout
        ]);

        let output = run_command_with_timeout(cmd, 30)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Transcoding(format!(
                "FFmpeg waveform extraction failed: {}",
                error
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

        // Reduzimos para um número fixo de pontos para o Frontend (ex: 500 pontos)
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

        // Normalização final (0.0 a 1.0)
        let max_amplitude = result.iter().fold(0.0f32, |max, &val| max.max(val));
        if max_amplitude > 0.0 {
            Ok(result.iter().map(|&v| v / max_amplitude).collect())
        } else {
            Ok(result)
        }
    }
}

/// Extensões de arquivos suportadas para áudio.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "m4a", "aac", "aiff", "wma", "mka", "ra", "mp2", "oga", "opus",
    "m4r", "spx", "ac3", "dts", "amr", "ape", "wv", "caf", "aax", "mid", "midi", "bwf",
];

/// Implementação do provedor de formato de áudio.
#[async_trait]
impl FormatProvider for AudioFormatProvider {
    /// Retorna o nome do provedor.
    ///
    /// # Returns
    ///
    /// * `&'static str` - O nome do provedor.
    fn name(&self) -> &'static str {
        "FFMPEG_AUDIO_PROVIDER"
    }

    /// Retorna as extensões de arquivos suportadas para áudio.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - As extensões de arquivos suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        AUDIO_EXTENSIONS.to_vec()
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "MP3 Audio",
                vec!["mp3", "mp2"],
                vec!["audio/mpeg"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "WAV Audio",
                vec!["wav", "bwf"],
                vec!["audio/wav", "audio/x-wav"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "FLAC Audio",
                vec!["flac"],
                vec!["audio/flac"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "OGG Audio",
                vec!["ogg", "oga", "ogv"],
                vec!["audio/ogg"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "MPEG-4 Audio",
                vec!["m4a", "m4r", "aac"],
                vec!["audio/mp4", "audio/aac"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "AIFF Audio",
                vec!["aiff", "caf"],
                vec!["audio/x-aiff"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "Windows Media Audio",
                vec!["wma"],
                vec!["audio/x-ms-wma"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "Opus Audio",
                vec!["opus"],
                vec!["audio/opus"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "MIDI Audio",
                vec!["mid", "midi"],
                vec!["audio/midi"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Matroska Audio",
                vec!["mka"],
                vec!["audio/x-matroska"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "Speex Audio",
                vec!["spx"],
                vec!["audio/x-speex"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "Monkey's Audio",
                vec!["ape"],
                vec!["audio/x-ape"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
            SupportedFormat::with_metadata(
                "WavPack Audio",
                vec!["wv"],
                vec!["audio/x-wavpack"],
                MediaType::Audio,
                PreviewStrategy::None,
                PlaybackStrategy::AudioHls,
            ),
        ]
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn MetadataCapability>` - O provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    // Áudio não possui ThumbnailCapability visual por padrão nesta sprint
    // (Pode usar IconFormatProvider como fallback)
}

/// Implementação do provedor de metadados.
#[async_trait]
impl MetadataCapability for AudioFormatProvider {
    /// Extrai metadados técnicos do áudio.
    ///
    /// Utiliza FFprobe para extração de metadados técnicos.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo de áudio.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados técnicos.
    async fn extract_technical(&self, path: &Path) -> AppResult<Value> {
        let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

        let mut cmd = Command::new(tools.ffprobe);
        cmd.args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
            &path.to_string_lossy(),
        ]);

        let output = run_command_with_timeout(cmd, 15)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Transcoding(format!("FFprobe failed: {}", error)));
        }

        let json: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| AppError::Transcoding(format!("Failed to parse FFprobe JSON: {}", e)))?;

        let mut technical = serde_json::Map::new();

        if let Some(format) = json.get("format") {
            if let Some(duration) = format.get("duration").and_then(|d| d.as_str()) {
                technical.insert("duration".to_string(), Value::String(duration.to_string()));
            }
            if let Some(bit_rate) = format.get("bit_rate").and_then(|b| b.as_str()) {
                technical.insert("bit_rate".to_string(), Value::String(bit_rate.to_string()));
            }
        }

        // Tenta pegar o codec da track de áudio
        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            for stream in streams {
                if stream.get("codec_type").and_then(|t| t.as_str()) == Some("audio") {
                    if let Some(codec) = stream.get("codec_name") {
                        technical.insert("codec".to_string(), codec.clone());
                    }
                    if let Some(sample_rate) = stream.get("sample_rate") {
                        technical.insert("sample_rate".to_string(), sample_rate.clone());
                    }
                    if let Some(channels) = stream.get("channels") {
                        technical.insert("channels".to_string(), channels.clone());
                    }
                }
            }
        }

        Ok(Value::Object(technical))
    }

    /// Extrai metadados semânticos do áudio.
    ///
    /// Atualmente, não há extração semântica nativa via FFmpeg para áudio.
    ///
    /// # Arguments
    ///
    /// * `_path` - O caminho para o arquivo de áudio.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados semânticos.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        // Futuro: Extração de letras ou tags ID3 avançadas
        Ok(serde_json::json!({}))
    }
}
