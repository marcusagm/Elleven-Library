use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for text and markdown formats.
#[derive(Default)]
pub struct TextFormatProvider;

impl TextFormatProvider {
    /// Create a new instance of `TextFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for TextFormatProvider {
    fn name(&self) -> &'static str {
        "TEXT_FORMAT_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "md", "log", "json", "xml", "csv"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Plain Text",
                vec!["txt", "log"],
                vec!["text/plain"],
                MediaType::Document,
                ThumbnailStrategy::Icon,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Markdown Document",
                vec!["md"],
                vec!["text/markdown"],
                MediaType::Document,
                ThumbnailStrategy::Icon,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Structured Data",
                vec!["json", "xml", "csv"],
                vec!["application/json", "application/xml", "text/csv"],
                MediaType::Document,
                ThumbnailStrategy::Icon,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
        ]
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for TextFormatProvider {
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let content = std::fs::read_to_string(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let lines = content.lines().count();
            let characters = content.chars().count();
            let words = content.split_whitespace().count();

            Ok(serde_json::json!({
                "lines": lines,
                "characters": characters,
                "words": words,
                "encoding": "UTF-8"
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for TextFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let extension = path_owned.extension().and_then(|e| e.to_str()).unwrap_or("txt").to_lowercase();

        let mime_type = match extension.as_str() {
            "md" => "text/markdown",
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            _ => "text/plain",
        };

        let data = tokio::fs::read(path_owned).await.map_err(crate::core::error::AppError::Io)?;
        Ok((data, mime_type.to_string()))
    }
}
