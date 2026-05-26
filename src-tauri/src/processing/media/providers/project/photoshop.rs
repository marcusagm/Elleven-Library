use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Adobe Photoshop files (.psd, .psb).
///
/// This provider handles both standard PSD files and Large Document Format (PSB) files,
/// extracting technical metadata like dimensions and color mode, as well as semantic
/// data like layer names.
///
/// # Technical Details
///
/// - **File Format**: PSD or PSB
/// - **Preview Format**: PNG image
/// - **Metadata**: JSON data containing design information
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::photoshop::PhotoshopFormatProvider;
///
/// let provider = PhotoshopFormatProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 2);
/// assert_eq!(formats[0].name, "Adobe Photoshop Image");
/// assert_eq!(formats[0].extensions, vec!["psd"]);
/// assert_eq!(formats[1].name, "Adobe Photoshop Large Image");
/// assert_eq!(formats[1].extensions, vec!["psb"]);
/// ```
#[derive(Default)]
pub struct PhotoshopFormatProvider;

impl PhotoshopFormatProvider {
    /// Creates a new instance of `PhotoshopFormatProvider`.
    ///
    /// # Returns
    ///
    /// `PhotoshopFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for PhotoshopFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "ADOBE_PHOTOSHOP_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["psd", "psb"]
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

        vec![
            SupportedFormat::with_metadata(
                "Adobe Photoshop Image",
                vec!["psd"],
                vec!["image/vnd.adobe.photoshop"],
                MediaType::Image,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Adobe Photoshop Large Image",
                vec!["psb"],
                vec!["image/vnd.adobe.photoshop"],
                MediaType::Image,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Validates if the file header matches the Photoshop magic bytes ("8BPS").
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid Photoshop file.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"8BPS")
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

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for PhotoshopFormatProvider {
    /// Extracts technical metadata such as width, height, and color mode.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Photoshop file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the PSD parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let photoshop_document = psd::Psd::from_bytes(&file_data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            Ok::<serde_json::Value, crate::core::error::AppError>(serde_json::json!({
                "width": photoshop_document.width(),
                "height": photoshop_document.height(),
                "color_mode": format!("{:?}", photoshop_document.color_mode()),
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata such as layer names.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Photoshop file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the PSD parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let photoshop_document = psd::Psd::from_bytes(&file_data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let layer_names: Vec<String> = photoshop_document
                .layers()
                .iter()
                .map(|layer| layer.name().to_string())
                .collect();

            Ok::<serde_json::Value, crate::core::error::AppError>(serde_json::json!({
                "layer_names": layer_names,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for PhotoshopFormatProvider {
    /// Generates a WebP thumbnail from the Photoshop composite image.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Photoshop file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Requested dimension for the thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the PSD parsing or image processing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let photoshop_document = psd::Psd::from_bytes(&file_data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let rgba_pixels = photoshop_document.rgba();

            let rgba_buffer = image::RgbaImage::from_raw(
                photoshop_document.width(),
                photoshop_document.height(),
                rgba_pixels,
            )
            .ok_or_else(|| {
                crate::core::error::AppError::Generic(
                    "Failed to create image buffer from PSD pixels".into(),
                )
            })?;
            let dynamic_image = image::DynamicImage::ImageRgba8(rgba_buffer);

            crate::processing::media::extractors::image::process_and_encode_webp(
                dynamic_image,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for PhotoshopFormatProvider {
    /// Generates a high-resolution PNG preview from the Photoshop composite image.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Photoshop file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the PSD parsing or image encoding fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let photoshop_document = psd::Psd::from_bytes(&file_data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let rgba_pixels = photoshop_document.rgba();
            let rgba_buffer = image::RgbaImage::from_raw(
                photoshop_document.width(),
                photoshop_document.height(),
                rgba_pixels,
            )
            .ok_or_else(|| {
                crate::core::error::AppError::Generic(
                    "Failed to create image buffer from PSD pixels".into(),
                )
            })?;
            let dynamic_image = image::DynamicImage::ImageRgba8(rgba_buffer);

            let mut output_buffer = std::io::Cursor::new(Vec::new());
            dynamic_image
                .write_to(&mut output_buffer, image::ImageFormat::Png)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            Ok::<(Vec<u8>, String), crate::core::error::AppError>((
                output_buffer.into_inner(),
                "image/png".to_string(),
            ))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
