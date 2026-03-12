use crate::core::error::AppResult;
use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;

/// Provider for PDF documents
#[derive(Default)]
pub struct PdfFormatProvider;

/// Implementation of `PdfFormatProvider`.
impl PdfFormatProvider {
    /// Create a new instance of `PdfFormatProvider`.
    ///
    /// # Returns
    ///
    /// A new instance of `PdfFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Trait for format provider.
impl FormatProvider for PdfFormatProvider {
    /// Get the name of the format provider.
    ///
    /// # Returns
    ///
    /// A `&'static str` containing the name of the format provider.
    fn name(&self) -> &'static str {
        "PDF_DOCUMENT_PROVIDER"
    }

    /// Get the supported extensions for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }

    /// Get the supported formats for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<SupportedFormat>` containing the supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Portable Document Format",
                vec!["pdf"],
                vec!["application/pdf"],
                MediaType::Image,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Check if the given header bytes support the format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes of the file.
    ///
    /// # Returns
    ///
    /// `true` if the format is supported, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"%PDF-")
    }

    /// Get the metadata capability for the format.
    ///
    /// # Returns
    ///
    /// An `Option<&dyn MetadataCapability>` containing the metadata capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }
}

/// Trait for metadata capability.
#[async_trait]
impl MetadataCapability for PdfFormatProvider {
    /// Extract technical metadata from the given PDF file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PDF file.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the technical metadata.
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        // Basic PDF metadata extraction (could use lopdf or pdf-extract later)
        Ok(serde_json::json!({
            "format": "PDF"
        }))
    }

    /// Extract semantic metadata from the given PDF file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PDF file.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the semantic metadata.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}
