use crate::core::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for PostScript / Encapsulated PostScript files (.eps, .ps).
#[derive(Default)]
pub struct PostscriptFormatProvider;

impl PostscriptFormatProvider {
    /// Creates a new instance of the PostScript format provider.
    ///
    /// # Returns
    ///
    /// `PostscriptFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for PostscriptFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "POSTSCRIPT_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["eps", "ps"]
    }

    /// Returns the list of supported formats.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Encapsulated PostScript",
                vec!["eps", "ps"],
                vec!["application/postscript"],
                MediaType::Vector,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Checks if the provider supports the given magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if the provider supports the given magic bytes.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"%!PS-Adobe")
    }

    /// Returns the metadata capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Returns the preview capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for PostscriptFormatProvider {
    /// Extracts technical metadata from the PostScript file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    ///
    /// # Returns
    ///
    /// `AppResult<Value>` - Technical metadata.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let metadata = extractors::extract_eps_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            Ok(metadata["technical"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the PostScript file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    ///
    /// # Returns
    ///
    /// `AppResult<Value>` - Semantic metadata.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for PostscriptFormatProvider {
    /// Generates a thumbnail from the PostScript file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    /// * `asset_id` - Identifier for the asset.
    /// * `size_hint` - Hint for the desired size.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail data.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file_extension = path_owned
                .extension()
                .and_then(|extension_os_string| extension_os_string.to_str())
                .unwrap_or("")
                .to_lowercase();
            let (preview_data, mime_type) = match file_extension.as_str() {
                "eps" | "ps" => extractors::extract_eps_ps_preview(&path_owned),
                _ => Err("Unsupported extension for PostScript provider".into()),
            }
            .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            if mime_type == "application/pdf" {
                extractors::render_pdf_to_png(&preview_data, size_hint)
                    .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
            } else {
                Ok(preview_data)
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for PostscriptFormatProvider {
    /// Generates a preview from the PostScript file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    /// * `asset_id` - Identifier for the asset.
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - Preview data and format.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the preview extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file_extension = path_owned
                .extension()
                .and_then(|extension_os_str| extension_os_str.to_str())
                .unwrap_or("")
                .to_lowercase();
            match file_extension.as_str() {
                "eps" | "ps" => extractors::extract_eps_ps_preview(&path_owned),
                _ => Err("Unsupported extension for PostScript provider".into()),
            }
            .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
