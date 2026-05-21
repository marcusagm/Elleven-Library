use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for DirectDraw Surface image files (.dds).
///
/// DDS is a Microsoft texture format used by games and 3D applications.
/// `image-rs` decodes most DDS variants (DXT1–DXT5, BC1–BC7). Preview uses
/// `Convert` strategy since browsers cannot render DDS natively.
///
/// # Technical Details
///
/// - **File Format**: DirectDraw Surface (DDS)
/// - **Thumbnail Format**: WebP image
/// - **Metadata**: Dimensions
///
/// # Features
///
/// - Extracts dimensions and codec information from DDS files via image-rs.
/// - Generates WebP thumbnails from DDS files via image-rs.
///
/// # Examples
///
/// ```rust
/// use mundam_core::formats::provider::FormatProvider;
/// use mundam_core::formats::types::MediaType;
///
/// let provider = DdsFormatProvider::new();
/// let formats = provider.supported_formats();
/// assert!(formats.contains(&MediaType::Image));
/// ```
#[derive(Default)]
pub struct DdsFormatProvider;

impl DdsFormatProvider {
    /// Creates a new instance of `DdsFormatProvider`.
    ///
    /// # Returns
    ///
    /// `DdsFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for DdsFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "DDS_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["dds"]
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
            "DirectDraw Surface",
            vec!["dds"],
            vec!["image/vnd-ms.dds"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::Convert,
            PlaybackStrategy::None,
        )]
    }

    /// Validates DDS magic bytes (`DDS `).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match DDS format, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"DDS ")
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
impl MetadataCapability for DdsFormatProvider {
    /// Extracts dimensions from a DDS file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the DDS file.
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

    /// Extracts semantic metadata from a DDS file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the DDS file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - JSON object with semantic metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be opened.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for DdsFormatProvider {
    /// Generates a WebP thumbnail from a DDS texture file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the DDS file.
    /// * `_asset_id` - The ID of the asset (unused in current implementation).
    /// * `size_hint` - The desired size of the thumbnail (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the generated thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the DDS variant is unsupported by `image-rs`.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_hdr_exr_dds_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for DdsFormatProvider {
    /// Generates a WebP preview from a DDS texture file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the DDS file.
    /// * `_asset_id` - The ID of the asset (unused in current implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the DDS variant is unsupported by `image-rs`.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::generate_hdr_exr_dds_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
