use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Web Open Font Format 2 files (.woff2).
///
/// This provider handles standard Web Open Font Format (WOFF) version 2 files,
/// extracting technical metadata such as family name, style properties,
/// and number of glyphs, as well as generating font preview thumbnails.
///
/// # Technical Details
///
/// - **File Format**: WOFF2
/// - **Preview/Thumbnail Format**: WebP image
/// - **Metadata**: JSON data containing font metrics and name records
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::font::woff2::Woff2FontProvider;
///
/// let provider = Woff2FontProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].name, "Web Open Font Format 2");
/// assert_eq!(formats[0].extensions, vec!["woff2"]);
/// ```
#[derive(Default)]
pub struct Woff2FontProvider;

impl Woff2FontProvider {
    /// Creates a new instance of `Woff2FontProvider`.
    ///
    /// # Returns
    ///
    /// `Woff2FontProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for Woff2FontProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "WOFF2_FONT_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["woff2"]
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
            "Web Open Font Format 2",
            vec!["woff2"],
            vec!["font/woff2", "application/font-woff2"],
            MediaType::Font,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::None,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches WOFF2 magic bytes ("wOF2").
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid WOFF2 file.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"wOF2")
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for Woff2FontProvider {
    /// Extracts technical metadata such as family name, weight, and metrics.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the WOFF2 font file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If font parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::font::extract_font_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata (not supported for fonts).
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for Woff2FontProvider {
    /// Generates a WebP thumbnail containing sample characters rendered in the font.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the WOFF2 font file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Requested dimension for the thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If font rendering fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::font::generate_font_thumbnail(&path_owned, size_hint)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
