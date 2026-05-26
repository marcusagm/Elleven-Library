use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for TrueType Collection files (.ttc).
///
/// This provider handles TrueType Collection (TTC) files,
/// which contain multiple font faces inside a single file,
/// extracting technical metadata for all faces as well as
/// generating font preview thumbnails from the primary face.
///
/// # Technical Details
///
/// - **File Format**: TTC
/// - **Preview/Thumbnail Format**: WebP image
/// - **Metadata**: JSON data containing font metrics for all contained faces
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::font::ttc::TrueTypeCollectionProvider;
///
/// let provider = TrueTypeCollectionProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].name, "TrueType Collection");
/// assert_eq!(formats[0].extensions, vec!["ttc"]);
/// ```
#[derive(Default)]
pub struct TrueTypeCollectionProvider;

impl TrueTypeCollectionProvider {
    /// Creates a new instance of `TrueTypeCollectionProvider`.
    ///
    /// # Returns
    ///
    /// `TrueTypeCollectionProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for TrueTypeCollectionProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "TRUETYPE_COLLECTION_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ttc"]
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
            "TrueType Collection",
            vec!["ttc"],
            vec![
                "font/collection",
                "font/ttc",
                "application/x-font-collection",
            ],
            MediaType::Font,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::None,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches TrueType Collection magic bytes ("ttcf").
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid TrueType Collection file.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"ttcf")
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
impl MetadataCapability for TrueTypeCollectionProvider {
    /// Extracts technical metadata such as family name, weight, and metrics for all faces.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TrueType Collection file.
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
impl ThumbnailCapability for TrueTypeCollectionProvider {
    /// Generates a WebP thumbnail containing sample characters rendered in the font.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TrueType Collection file.
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
            crate::processing::media::extractors::font::generate_font_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
