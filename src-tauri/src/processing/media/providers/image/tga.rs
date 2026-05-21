use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Truevision TGA image files (.tga).
///
/// TGA (TARGA) is a legacy raster format used in game development. It lacks
/// a standard magic byte header (the format identifier is at the end of the
/// file), so magic-byte detection is disabled. Extension matching is primary.
/// Preview uses `Convert` since browsers cannot render TGA natively.
///
/// # Note
///
/// The absence of a leading magic byte is the historical reason TGA files are
/// sometimes misidentified. This provider relies on extension matching only.
///
/// # Examples
///
/// ```rust
/// use mundam_lib::processing::media::providers::image::tga::TgaFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
/// use mundam_lib::core::formats::types::MediaType;
///
/// let provider = TgaFormatProvider::new();
/// let formats = provider.supported_formats();
/// assert!(formats.contains(&MediaType::Image));
/// ```
#[derive(Default)]
pub struct TgaFormatProvider;

impl TgaFormatProvider {
    /// Creates a new instance of `TgaFormatProvider`.
    ///
    /// # Returns
    ///
    /// `TgaFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for TgaFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "TGA_IMAGE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["tga"]
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
            "Truevision TGA Image",
            vec!["tga"],
            vec!["image/x-tga", "image/x-targa"],
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
    /// `bool` - `false` as TGA has no mandatory magic bytes.
    fn supports_magic_bytes(&self, _header_bytes: &[u8]) -> bool {
        false
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
impl MetadataCapability for TgaFormatProvider {
    /// Extracts dimensions from a TGA file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.tga` file.
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

    /// Extracts semantic metadata from the TGA file.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the `.tga` file (not used in this implementation).
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
impl ThumbnailCapability for TgaFormatProvider {
    /// Generates a WebP thumbnail from a TGA file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.tga` file.
    /// * `asset_id` - The asset ID for the file.
    /// * `size_hint` - Hint for the desired size of the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Vector of bytes representing the WebP thumbnail.
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
impl PreviewCapability for TgaFormatProvider {
    /// Generates a preview from a TGA file using image-rs.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.tga` file.
    /// * `_asset_id` - Asset ID (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If TGA decoding or encoding fails.
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
