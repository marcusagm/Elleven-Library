use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Aseprite files (.ase, .aseprite).
///
/// This provider uses the `asefile` crate to parse Aseprite's proprietary binary format,
/// extracting technical metadata (width, height, frames, layers) and semantic data
/// (layer names). Preview generation is handled by a specialized internal extractor.
///
/// # Technical Details
///
/// - **File Format**: Aseprite
/// - **Preview Format**: PNG or GIF (animated)
/// - **Metadata**: JSON data containing design information
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::aseprite::AsepriteFormatProvider;
///
/// let provider = AsepriteFormatProvider::new();
/// let formats = provider.supported_formats();
///
/// assert_eq!(formats.len(), 1);
/// assert_eq!(formats[0].name, "Aseprite Sprite");
/// assert_eq!(formats[0].extensions, vec!["ase", "aseprite"]);
/// ```
#[derive(Default)]
pub struct AsepriteFormatProvider;

impl AsepriteFormatProvider {
    /// Creates a new instance of `AsepriteFormatProvider`.
    ///
    /// # Returns
    ///
    /// `AsepriteFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for AsepriteFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "ASEPRITE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ase", "aseprite"]
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
            "Aseprite Sprite",
            vec!["ase", "aseprite"],
            vec!["image/x-aseprite"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches the Aseprite magic identifier (0xA5E0 at offset 4).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid Aseprite file.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        if header_bytes.len() < 6 {
            return false;
        }
        // Aseprite files start with the file size (4 bytes) followed by magic 0xA5E0 (2 bytes)
        header_bytes[4] == 0xE0 && header_bytes[5] == 0xA5
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
impl MetadataCapability for AsepriteFormatProvider {
    /// Extracts technical metadata such as dimensions, frame count, and layer count.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Aseprite file.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the Aseprite file parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let aseprite_file = asefile::AsepriteFile::read_file(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            Ok(serde_json::json!({
                "width": aseprite_file.width(),
                "height": aseprite_file.height(),
                "num_frames": aseprite_file.num_frames(),
                "num_layers": aseprite_file.num_layers(),
                "color_depth": format!("{:?}", aseprite_file.pixel_format()),
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata such as the names of all layers in the file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Aseprite file.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If the Aseprite file parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let aseprite_file = asefile::AsepriteFile::read_file(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let layer_names: Vec<String> = (0..aseprite_file.num_layers())
                .map(|layer_index| aseprite_file.layer(layer_index).name().to_string())
                .collect();

            Ok(serde_json::json!({
                "layer_names": layer_names,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for AsepriteFormatProvider {
    /// Generates a thumbnail for the Aseprite file by rendering the first frame.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Aseprite file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Requested dimension for the thumbnail (ignored for native previews).
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction or rendering fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_aseprite_preview(&path_owned)
                .map(|(image_data, _mime_type)| image_data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for AsepriteFormatProvider {
    /// Generates a high-resolution PNG preview of the Aseprite file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Aseprite file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction or rendering fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_aseprite_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
