use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Portable AnyMap image files (.pbm, .pgm, .ppm, .pnm, .pam).
///
/// The NetPBM family covers a group of related plain-text and binary image
/// formats. All variants are decoded by `image-rs`. Preview uses `Convert`
/// since browsers cannot render these formats natively.
///
/// # Supported Extensions
///
/// | Extension | Full Name                    |
/// |-----------|------------------------------|
/// | `pbm`     | Portable Bit Map             |
/// | `pgm`     | Portable Gray Map            |
/// | `ppm`     | Portable Pixel Map           |
/// | `pnm`     | Portable Any Map (generic)   |
/// | `pam`     | Portable Arbitrary Map       |
///
/// # Technical Details
///
/// - **File Format**: NetPBM (PBM, PGM, PPM, PAM)
/// - **Thumbnail Format**: WebP image
/// - **Metadata**: Dimensions
///
/// # Examples
///
/// ```rust
/// use mundam_lib::processing::media::providers::image::netpbm::NetpbmFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
///
/// let provider = NetpbmFormatProvider::new();
/// let formats = provider.supported_formats();
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].display_name, "Portable AnyMap Image");
/// ```
#[derive(Default)]
pub struct NetpbmFormatProvider;

impl NetpbmFormatProvider {
    /// Creates a new instance of `NetpbmFormatProvider`.
    ///
    /// # Returns
    ///
    /// `NetpbmFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for NetpbmFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "NETPBM_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["pbm", "pgm", "ppm", "pnm", "pam"]
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
            "Portable AnyMap Image",
            vec!["pbm", "pgm", "ppm", "pnm", "pam"],
            vec![
                "image/x-portable-bitmap",
                "image/x-portable-graymap",
                "image/x-portable-pixmap",
                "image/x-portable-anymap",
            ],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::Convert,
            PlaybackStrategy::None,
        )]
    }

    /// Validates NetPBM magic bytes (ASCII `P1`–`P6` or binary `P7`).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match NetPBM magic bytes, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        matches!(
            header_bytes.get(..2),
            Some(b"P1")
                | Some(b"P2")
                | Some(b"P3")
                | Some(b"P4")
                | Some(b"P5")
                | Some(b"P6")
                | Some(b"P7")
        )
    }

    /// Returns the preview capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview capability for this provider.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }

    /// Returns the metadata capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata capability for this provider.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail capability for this provider.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for NetpbmFormatProvider {
    /// Extracts dimensions from a NetPBM file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the NetPBM file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - JSON object with technical metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be opened.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_raster_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from a NetPBM file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the NetPBM file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - JSON object with semantic metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    /// * `AppError::Generic` - If the metadata extraction fails.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for NetpbmFormatProvider {
    /// Generates a WebP thumbnail from a NetPBM file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the NetPBM file.
    /// * `asset_id` - The asset ID for the file.
    /// * `size_hint` - Requested maximum dimension in pixels.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - The generated thumbnail as a byte vector.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If decoding or encoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_raster_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for NetpbmFormatProvider {
    /// Generates a WebP preview from a NetPBM file using image-rs.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the NetPBM file.
    /// * `_asset_id` - Asset ID (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If decoding or encoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_raster_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
