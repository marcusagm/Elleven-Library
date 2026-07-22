use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for structured data files (.json, .xml, .csv).
///
/// Handles machine-readable text formats, extracting text statistics as
/// technical metadata and serving the raw file content with the correct
/// MIME type for browser-side preview.
///
/// # Technical Details
///
/// - **File Format**: JSON, XML, or CSV
/// - **Preview Format**: Raw text served with format-specific MIME type
/// - **Metadata**: Line, character, and word counts
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::document::structured_data::StructuredDataFormatProvider;
///
/// let provider = StructuredDataFormatProvider::new();
/// assert_eq!(provider.supported_formats()[0].extensions, vec!["json", "xml", "csv"]);
/// ```
#[derive(Default)]
pub struct StructuredDataFormatProvider;

impl StructuredDataFormatProvider {
    /// Creates a new instance of `StructuredDataFormatProvider`.
    ///
    /// # Returns
    ///
    /// `StructuredDataFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for StructuredDataFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "STRUCTURED_DATA_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["json", "xml", "csv"]
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
            "Structured Data",
            vec!["json", "xml", "csv"],
            vec!["application/json", "application/xml", "text/csv"],
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
impl MetadataCapability for StructuredDataFormatProvider {
    /// Extracts text statistics: line count, character count, and word count.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the structured data file.
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

    /// Returns empty semantic metadata (not implemented for structured data).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for StructuredDataFormatProvider {
    /// Serves the raw file content with the correct MIME type based on extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the structured data file.
    /// * `_asset_id` - The ID of the asset (unused).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - File bytes and format-specific MIME type.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let extension = path_owned
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("json")
            .to_lowercase();

        let mime_type = match extension.as_str() {
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            _ => "application/octet-stream",
        };

        let data = tokio::fs::read(path_owned)
            .await
            .map_err(crate::core::error::AppError::Io)?;
        Ok((data, mime_type.to_string()))
    }
}
