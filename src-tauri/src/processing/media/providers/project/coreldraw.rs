//! CorelDRAW (.cdr) format provider.
//!
//! Provides metadata, thumbnail, and preview extraction for all CorelDRAW
//! versions (v3 to modern ZIP-based v24+).

use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::formats::types::{
    MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
};
use crate::core::AppResult;
use crate::processing::media::extractors;
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::instrument;

/// Provider for CorelDRAW (.cdr) files.
///
/// This provider uses the `extract-coreldraw` crate to extract metadata, thumbnails, and previews
/// from CorelDRAW files. CorelDRAW files are ZIP archives containing JSON data and previews.
///
/// # Technical Details
///
/// - **File Format**: ZIP archive
/// - **Preview Format**: PNG image
/// - **Metadata**: JSON data containing design information
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::coreldraw::CoreldrawFormatProvider;
///
/// let provider = CoreldrawFormatProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].name, "CorelDRAW Drawing");
/// assert_eq!(formats[0].extensions, vec!["cdr"]);
/// ```
#[derive(Default)]
pub struct CoreldrawFormatProvider;

impl CoreldrawFormatProvider {
    /// Creates a new instance of the CorelDRAW format provider.
    ///
    /// # Returns
    ///
    /// `CoreldrawFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for CoreldrawFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "CORELDRAW_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cdr"]
    }

    /// Returns the list of supported formats.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![SupportedFormat::with_metadata(
            "CorelDRAW Drawing",
            vec!["cdr"],
            vec!["application/x-coreldraw", "application/vnd.corel-draw"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
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
        // Modern ZIP: PK..
        header_bytes.starts_with(b"PK\x03\x04") ||
        // Legacy RIFF: RIFF....CDR
        (header_bytes.starts_with(b"RIFF") && header_bytes.len() >= 12 &&
         (&header_bytes[8..11] == b"CDR" || &header_bytes[8..11] == b"cdr")) ||
        // Ancient WL: WL
        header_bytes.starts_with(b"WL")
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
impl ThumbnailCapability for CoreldrawFormatProvider {
    /// Generates a thumbnail from the CorelDRAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CorelDRAW file.
    /// * `asset_id` - Identifier for the asset.
    /// * `size_hint` - Hint for the desired size.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail data.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the thumbnail extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_coreldraw_preview(&path_owned)
                .map(|(data, _)| data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for CoreldrawFormatProvider {
    /// Generates a preview from the CorelDRAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CorelDRAW file.
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
            extractors::extract_coreldraw_preview_highres(&path_owned, 2048)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for CoreldrawFormatProvider {
    /// Extracts technical metadata from the CorelDRAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the CorelDRAW file.
    ///
    /// # Returns
    ///
    /// `AppResult<Value>` - Technical metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the metadata extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_coreldraw_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}
