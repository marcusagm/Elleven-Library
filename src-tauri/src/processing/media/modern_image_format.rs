//! Provedor para formatos de imagem modernos (HEIC, AVIF, JXL).
//!
//! Estes formatos são delegados ao FFmpeg para garantir performance de decodificação
//! e suporte a HDR em contêineres HEIF/AVIF.

use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

/// Implementação do provedor de formatos modernos de imagem.
pub struct ModernImageFormatProvider {}

/// Implementação do provedor de formatos modernos de imagem.
impl Default for ModernImageFormatProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ModernImageFormatProvider {
    pub fn new() -> Self {
        Self {}
    }
}

/// Extensões suportadas para formatos modernos de imagem.
pub const MODERN_EXTENSIONS: &[&str] = &["heic", "heif", "avif", "jxl"];

/// Implementação do provedor de formatos modernos de imagem.
#[async_trait]
impl FormatProvider for ModernImageFormatProvider {
    /// Retorna o nome do provedor.
    ///
    /// # Returns
    ///
    /// * `&'static str` - O nome do provedor.
    fn name(&self) -> &'static str {
        "FFMPEG_MODERN_IMAGE_PROVIDER"
    }

    /// Retorna as extensões suportadas.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - As extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        MODERN_EXTENSIONS.to_vec()
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
impl MetadataCapability for ModernImageFormatProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo.
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

        let output = run_command_with_timeout(cmd, 10)?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Transcoding(format!("FFprobe failed: {}", error)));
        }

        let json: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| AppError::Transcoding(format!("Failed to parse FFprobe JSON: {}", e)))?;

        let mut technical = serde_json::Map::new();

        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            if let Some(stream) = streams.first() {
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

        Ok(Value::Object(technical))
    }

    /// Extrai metadados semânticos do arquivo.
    ///
    /// # Arguments
    ///
    /// * `_path` - O caminho para o arquivo.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados semânticos.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}

/// Implementação do provedor de thumbnails.
#[async_trait]
impl ThumbnailCapability for ModernImageFormatProvider {
    /// Gera um thumbnail para o arquivo.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo.
    /// * `size_hint` - O tamanho do thumbnail.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - O resultado da geração do thumbnail.
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

        let mut cmd = Command::new(tools.ffmpeg);
        cmd.args([
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

        let output = run_command_with_timeout(cmd, 15)?;

        if output.status.success() {
            Ok(output.stdout)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Transcoding(format!(
                "FFmpeg image extraction failed: {}",
                error
            )))
        }
    }
}
