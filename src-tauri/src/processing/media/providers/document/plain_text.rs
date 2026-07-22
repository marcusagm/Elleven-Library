use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for plain text files (.txt, .log).
///
/// Extracts text statistics (line count, character count, word count) as
/// technical metadata and serves the raw file content as a browser-native
/// preview.
///
/// # Technical Details
///
/// - **File Format**: Plain text (UTF-8 assumed)
/// - **Preview Format**: Raw text served as `text/plain`
/// - **Metadata**: Line, character, and word counts
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::document::plain_text::PlainTextFormatProvider;
///
/// let provider = PlainTextFormatProvider::new();
/// assert_eq!(provider.supported_formats()[0].extensions, vec!["txt", "log"]);
/// ```
#[derive(Default)]
pub struct PlainTextFormatProvider;

impl PlainTextFormatProvider {
    /// Creates a new instance of `PlainTextFormatProvider`.
    ///
    /// # Returns
    ///
    /// `PlainTextFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for PlainTextFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "PLAIN_TEXT_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "log"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats with metadata.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "Plain Text",
            vec!["txt", "log"],
            vec!["text/plain"],
            MediaType::Document,
            ThumbnailStrategy::Icon,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::None,
        )]
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for PlainTextFormatProvider {
    /// Extracts text statistics: line count, character count, and word count.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the text file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let content =
                std::fs::read_to_string(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let line_count = content.lines().count();
            let character_count = content.chars().count();
            let word_count = content.split_whitespace().count();

            Ok(serde_json::json!({
                "lines": line_count,
                "characters": character_count,
                "words": word_count,
                "encoding": "UTF-8"
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Returns empty semantic metadata (not applicable for plain text).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the text file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for PlainTextFormatProvider {
    /// Serves the raw file content as a plain text preview.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the text file.
    /// * `_asset_id` - The ID of the asset (unused).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - File bytes and MIME type `text/plain`.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let data = tokio::fs::read(path_owned)
            .await
            .map_err(crate::core::error::AppError::Io)?;
        Ok((data, "text/plain".to_string()))
    }
}
