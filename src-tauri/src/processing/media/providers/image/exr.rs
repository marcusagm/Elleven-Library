use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for OpenEXR image files (.exr).
///
/// OpenEXR is an HDR image format developed by Industrial Light & Magic,
/// widely used in VFX and film production. The `image-rs` crate handles
/// decoding. Preview uses `Convert` strategy since browsers cannot render
/// EXR natively.
///
/// # Technical Details
///
/// - **File Format**: OpenEXR
/// - **Thumbnail Format**: WebP image
/// - **Metadata**: Dimensions and colour mode
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::image::exr::ExrFormatProvider;
///
/// let provider = ExrFormatProvider::new();
/// assert_eq!(provider.supported_formats()[0].extensions, vec!["exr"]);
/// ```
#[derive(Default)]
pub struct ExrFormatProvider;

impl ExrFormatProvider {
    /// Creates a new instance of `ExrFormatProvider`.
    ///
    /// # Returns
    ///
    /// `ExrFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for ExrFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "OPENEXR_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["exr"]
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
            "OpenEXR Image",
            vec!["exr"],
            vec!["image/x-exr"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::Convert,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the provider supports the given magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match OpenEXR magic bytes, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"\x76\x2F\x31\x01")
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
impl MetadataCapability for ExrFormatProvider {
    /// Extracts dimensions and colour mode from an OpenEXR file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.exr` file.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If EXR decoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_exr_metadata(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from the OpenEXR file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the `.exr` file (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - An empty JSON object.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If any error occurs during semantic metadata extraction.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for ExrFormatProvider {
    /// Generates a WebP thumbnail from an OpenEXR file with tone-mapping via `image-rs`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.exr` file.
    /// * `size_hint` - Requested maximum dimension in pixels.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If EXR decoding or encoding fails.
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
impl PreviewCapability for ExrFormatProvider {
    /// Generates a preview from an OpenEXR file with tone-mapping via `image-rs`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.exr` file.
    /// * `_asset_id` - Asset ID (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If EXR decoding or encoding fails.
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
