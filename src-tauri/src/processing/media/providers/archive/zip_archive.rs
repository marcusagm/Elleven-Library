use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for ZIP archive files (.zip).
///
/// Generates thumbnails by searching for embedded preview images inside
/// the archive (e.g. `preview.png`, `QuickLook/Preview.png`).
///
/// # Technical Details
///
/// - **File Format**: ZIP (PK header)
/// - **Thumbnail**: Extracted from embedded preview images
/// - **Metadata**: Not supported
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::archive::zip_archive::ZipArchiveProvider;
///
/// let provider = ZipArchiveProvider::new();
/// assert_eq!(provider.supported_formats()[0].extensions, vec!["zip"]);
/// ```
#[derive(Default)]
pub struct ZipArchiveProvider;

impl ZipArchiveProvider {
    /// Creates a new instance of `ZipArchiveProvider`.
    ///
    /// # Returns
    ///
    /// `ZipArchiveProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for ZipArchiveProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "ZIP_ARCHIVE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["zip"]
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
            "ZIP Archive",
            vec!["zip"],
            vec!["application/zip"],
            MediaType::Archive,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates the ZIP magic bytes (`PK\x03\x04`).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header matches the ZIP signature.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"PK\x03\x04")
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
impl ThumbnailCapability for ZipArchiveProvider {
    /// Generates a thumbnail by extracting an embedded preview image from the ZIP.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ZIP archive.
    /// * `_asset_id` - The ID of the asset (unused).
    /// * `size_hint` - The desired dimension for the thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If no preview is found or decoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::archive::extract_zip_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
