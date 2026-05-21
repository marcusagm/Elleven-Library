use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Sigma RAW image files (.x3f).
///
/// X3F is Sigma's proprietary RAW format based on their Foveon X3 sensor
/// technology. The three-layer sensor captures red, green, and blue at every
/// pixel site, making X3F files larger than typical RAW files.
///
/// # Technical Details
///
/// * **File Format**: Sigma RAW Image (.x3f)
/// * **Thumbnail Format**: WebP image (via libraw)
/// * **Metadata**: Dimensions and camera settings via libraw
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::image::sigma::SigmaRawFormatProvider;
///
/// let provider = SigmaRawFormatProvider::new();
/// assert!(provider.supported_extensions().contains(&"x3f"));
/// ```
#[derive(Default)]
pub struct SigmaRawFormatProvider;

impl SigmaRawFormatProvider {
    /// Creates a new instance of `SigmaRawFormatProvider`.
    ///
    /// # Returns
    ///
    /// `Self` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for SigmaRawFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "SIGMA_RAW_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["x3f"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - The list of supported formats with their details.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};
        vec![SupportedFormat::with_metadata(
            "Sigma RAW Image",
            vec!["x3f"],
            vec!["image/x-sigma-x3f"],
            MediaType::Image,
            ThumbnailStrategy::Raw,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Checks if the header bytes correspond to a valid Sigma RAW file.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes correspond to a valid Sigma RAW file, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"FOVb")
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
impl MetadataCapability for SigmaRawFormatProvider {
    /// Extracts technical metadata from a Sigma RAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Sigma RAW file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - The technical metadata extracted from the Sigma RAW file.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    /// * `AppError::Generic` - If the metadata extraction fails.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || crate::processing::media::extractors::image::extract_raw_metadata(&path_owned))
            .await.map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata from a Sigma RAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Sigma RAW file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - The semantic metadata extracted from the Sigma RAW file.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for SigmaRawFormatProvider {
    /// Generates a thumbnail from a Sigma RAW file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the Sigma RAW file.
    /// * `asset_id` - The ID of the asset.
    /// * `size_hint` - The size hint for the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - The thumbnail generated from the Sigma RAW file.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    /// * `AppError::Generic` - If the thumbnail generation fails.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || crate::processing::media::extractors::image::generate_raw_thumbnail(&path_owned, size_hint))
            .await.map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for SigmaRawFormatProvider {
    /// Generates a WebP preview using LibRaw embedded preview extraction or brute-force scanning.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to generate a preview from.
    /// * `_asset_id` - The ID of the asset.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the preview image.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If all extraction tiers fail.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let bytes = tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::image::extract_raw_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)??;

        Ok((bytes, "image/jpeg".to_string()))
    }
}
