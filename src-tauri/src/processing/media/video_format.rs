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
#[derive(Default)]
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
    "mp4", "m4v", "webm", "mov", "qt", "mkv", "mxf", "wmv", "asf", "flv", "f4v", "swf", "mpg",
    "mpeg", "m2v", "vob", "ts", "mts", "m2ts", "avi", "divx", "3gp", "3g2", "rm", "rmvb", "wtv",
    "ogv", "mjpeg", "mjpg", "hevc", "h264", "h265", "y4m",
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

    fn supported_formats(&self) -> Vec<crate::core::formats::provider::SupportedFormat> {
        use crate::core::formats::provider::SupportedFormat;
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "MPEG-4 Video",
                vec!["mp4", "m4v"],
                vec!["video/mp4"],
                MediaType::Video,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "WebM Video",
                vec!["webm"],
                vec!["video/webm"],
                MediaType::Video,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "QuickTime Video",
                vec!["mov", "qt"],
                vec!["video/quicktime"],
                MediaType::Video,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::Native,
            ),
            SupportedFormat::with_metadata(
                "Matroska Video",
                vec!["mkv"],
                vec!["video/x-matroska"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Material Exchange Format",
                vec!["mxf"],
                vec!["video/mxf"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Windows Media Video",
                vec!["wmv", "asf"],
                vec!["video/x-ms-wmv", "video/x-ms-asf"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Flash Video",
                vec!["flv", "f4v"],
                vec!["video/x-flv"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Shockwave Flash",
                vec!["swf"],
                vec!["application/x-shockwave-flash"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
            ),
            SupportedFormat::with_metadata(
                "MPEG-1/2 Video",
                vec!["mpg", "mpeg", "m2v"],
                vec!["video/mpeg"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
            ),
            SupportedFormat::with_metadata(
                "MPEG Transport Stream",
                vec!["vob", "ts", "mts", "m2ts"],
                vec!["video/mp2t"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "AVI Video",
                vec!["avi", "divx"],
                vec!["video/x-msvideo"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "3GPP Video",
                vec!["3gp", "3g2"],
                vec!["video/3gpp"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "RealMedia Video",
                vec!["rm", "rmvb"],
                vec!["application/vnd.rn-realmedia"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Windows Recorded TV",
                vec!["wtv"],
                vec!["video/x-wtv"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Ogg Video",
                vec!["ogv"],
                vec!["video/ogg"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::Hls,
            ),
            SupportedFormat::with_metadata(
                "Motion JPEG",
                vec!["mjpeg", "mjpg"],
                vec!["video/x-motion-jpeg"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
            ),
            SupportedFormat::with_metadata(
                "HEVC Video",
                vec!["hevc", "h265"],
                vec!["video/hevc"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
            ),
            SupportedFormat::with_metadata(
                "H.264 Raw Video",
                vec!["h264"],
                vec!["video/h264"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
            ),
            SupportedFormat::with_metadata(
                "YUV4MPEG2 Video",
                vec!["y4m"],
                vec!["video/x-y4m"],
                MediaType::Video,
                PreviewStrategy::Ffmpeg,
                PlaybackStrategy::LinearHls,
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

        // Simplifica o JSON para o formato esperado pelo Mundam (VideoProbeResult)
        let mut technical = serde_json::Map::new();

        let mut video_codec = None;
        let mut audio_codec = None;
        let mut width = None;
        let mut height = None;
        let mut duration_secs = 0.0;
        let mut container = None;

        if let Some(format) = json.get("format") {
            if let Some(duration_str) = format.get("duration").and_then(|d| d.as_str()) {
                duration_secs = duration_str.parse::<f64>().unwrap_or(0.0);
            }
            container = format.get("format_name").and_then(|v| v.as_str()).map(|s| s.to_string());
        }

        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            for stream in streams {
                let codec_type = stream.get("codec_type").and_then(|t| t.as_str());
                let codec_name = stream.get("codec_name").and_then(|c| c.as_str()).map(|s| s.to_string());

                match codec_type {
                    Some("video") if video_codec.is_none() => {
                        video_codec = codec_name;
                        width = stream.get("width").and_then(|w| w.as_i64());
                        height = stream.get("height").and_then(|h| h.as_i64());
                    }
                    Some("audio") if audio_codec.is_none() => {
                        audio_codec = codec_name;
                    }
                    _ => {}
                }
            }
        }

        // Logic for is_native (V1 Parity)
        // Check if video and audio codecs are natively supported in modern WebView (WebKit/Blink)
        let native_video = match video_codec.as_deref() {
            Some("h264") | Some("avc1") | Some("avc") | Some("vp8") => true,
            None => true, // Still or audio only
            _ => false,
        };

        let native_audio = match audio_codec.as_deref() {
            Some("aac") | Some("mp3") | Some("mp2") | Some("flac") | Some("opus") | Some("vorbis") => true,
            Some(c) if c.starts_with("pcm_") => true,
            None => true,
            _ => false,
        };

        let is_native = native_video && native_audio;

        technical.insert("duration_secs".to_string(), serde_json::json!(duration_secs));
        technical.insert("video_codec".to_string(), serde_json::json!(video_codec));
        technical.insert("audio_codec".to_string(), serde_json::json!(audio_codec));
        technical.insert("container".to_string(), serde_json::json!(container));
        technical.insert("width".to_string(), serde_json::json!(width));
        technical.insert("height".to_string(), serde_json::json!(height));
        technical.insert("is_native".to_string(), serde_json::json!(is_native));

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
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
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
