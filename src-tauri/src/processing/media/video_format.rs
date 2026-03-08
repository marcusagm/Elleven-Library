//! Provedor de formato de vídeo.
//!
//! Utiliza FFprobe para extração de metadados técnicos e FFmpeg para
//! geração de thumbnails (frame grabbing).

use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Provedor de formato de vídeo.
pub struct VideoFormatProvider {}

/// Implementação do provedor de formato de vídeo.
impl VideoFormatProvider {
    /// Cria uma nova instância do provedor.
    ///
    /// # Returns
    ///
    /// * `Self` - A nova instância do provedor.
    pub fn new() -> Self {
        Self {}
    }
}

/// Extensões de arquivos suportadas para vídeo.
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "mov", "webm", "avi", "wmv", "flv", "m4v", "mxf", "asf", "ts", "mts", "m2ts",
    "vob", "3gp", "rm", "ogv", "swf", "mpg", "mpeg", "m2v", "divx", "h264", "h265", "hevc", "y4m",
    "mjpeg", "mjpg",
];

/// Implementação do provedor de formato de vídeo.
#[async_trait]
impl FormatProvider for VideoFormatProvider {
    /// Retorna o nome do provedor.
    ///
    /// # Returns
    ///
    /// * `&'static str` - O nome do provedor.
    fn name(&self) -> &'static str {
        "FFMPEG_VIDEO_PROVIDER"
    }

    /// Retorna as extensões de arquivos suportadas para vídeo.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - As extensões de arquivos suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        VIDEO_EXTENSIONS.to_vec()
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn MetadataCapability>` - O provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Retorna o provedor de thumbnails.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn ThumbnailCapability>` - O provedor de thumbnails.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementação do provedor de metadados.
#[async_trait]
impl MetadataCapability for VideoFormatProvider {
    /// Extrai metadados técnicos do vídeo.
    ///
    /// Utiliza FFprobe para extração de metadados técnicos.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo de vídeo.
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

        // Simplifica o JSON para o formato esperado pelo Mundam
        let mut technical = serde_json::Map::new();

        if let Some(format) = json.get("format") {
            if let Some(duration) = format.get("duration").and_then(|d| d.as_str()) {
                technical.insert("duration".to_string(), Value::String(duration.to_string()));
            }
            if let Some(bit_rate) = format.get("bit_rate").and_then(|b| b.as_str()) {
                technical.insert("bit_rate".to_string(), Value::String(bit_rate.to_string()));
            }
        }

        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            for stream in streams {
                if stream.get("codec_type").and_then(|t| t.as_str()) == Some("video") {
                    if let Some(width) = stream.get("width") {
                        technical.insert("width".to_string(), width.clone());
                    }
                    if let Some(height) = stream.get("height") {
                        technical.insert("height".to_string(), height.clone());
                    }
                    if let Some(codec) = stream.get("codec_name") {
                        technical.insert("codec".to_string(), codec.clone());
                    }
                }
            }
        }

        Ok(Value::Object(technical))
    }

    /// Extrai metadados semânticos do vídeo.
    ///
    /// Atualmente, não há extração semântica nativa via FFmpeg para vídeo.
    ///
    /// # Arguments
    ///
    /// * `_path` - O caminho para o arquivo de vídeo.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados semânticos.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        // Vídeos não possuem extração semântica nativa via FFmpeg por enquanto
        Ok(serde_json::json!({}))
    }
}

/// Implementação do provedor de thumbnails.
#[async_trait]
impl ThumbnailCapability for VideoFormatProvider {
    /// Gera um thumbnail para o vídeo.
    ///
    /// Utiliza FFmpeg para geração de thumbnail.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo de vídeo.
    /// * `size_hint` - O tamanho desejado para o thumbnail.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - O resultado da geração do thumbnail.
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

        // Tenta capturar no segundo 1 para evitar telas pretas iniciais
        let mut cmd = Command::new(&tools.ffmpeg);
        cmd.args([
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

        let output = run_command_with_timeout(cmd, 15)?;

        if output.status.success() {
            return Ok(output.stdout);
        }

        // Se falhar no segundo 1 (ex: vídeo curto), tenta no segundo 0
        let mut retry_cmd = Command::new(&tools.ffmpeg);
        retry_cmd.args([
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

        let retry_output = run_command_with_timeout(retry_cmd, 15)?;

        if retry_output.status.success() {
            Ok(retry_output.stdout)
        } else {
            let error = String::from_utf8_lossy(&retry_output.stderr);
            Err(AppError::Transcoding(format!(
                "FFmpeg frame extraction failed: {}",
                error
            )))
        }
    }
}
