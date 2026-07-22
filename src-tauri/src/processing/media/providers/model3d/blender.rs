use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Blender project files (.blend).
///
/// Extracts the embedded JPEG thumbnail from the Blender REND block for
/// thumbnail generation. Full metadata extraction is not yet implemented.
///
/// # Technical Details
///
/// - **File Format**: Blender (magic bytes: `BLENDER`)
/// - **Thumbnail**: Embedded JPEG in REND block
/// - **Preview**: Native extraction
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::model3d::blender::BlenderFormatProvider;
///
/// let provider = BlenderFormatProvider::new();
/// assert_eq!(provider.supported_formats()[0].extensions, vec!["blend"]);
/// ```
#[derive(Default)]
pub struct BlenderFormatProvider;

impl BlenderFormatProvider {
    /// Creates a new instance of `BlenderFormatProvider`.
    ///
    /// # Returns
    ///
    /// `BlenderFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for BlenderFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "BLENDER_PROJECT_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["blend"]
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
            "Blender Project",
            vec!["blend"],
            vec!["application/x-blender"],
            MediaType::Model3D,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates the Blender magic bytes (`BLENDER`).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header matches the Blender signature.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"BLENDER")
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
impl MetadataCapability for BlenderFormatProvider {
    /// Returns empty technical metadata (not yet implemented for Blender).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the Blender file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    /// Returns empty semantic metadata (not yet implemented for Blender).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the Blender file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for BlenderFormatProvider {
    /// Extracts the embedded JPEG thumbnail from the Blender REND block.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Blender file.
    /// * `_asset_id` - The ID of the asset (unused).
    /// * `_size_hint` - The desired dimension (unused, thumbnail is pre-sized).
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::FormatNotSupported` - If no thumbnail is found in the file.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::model3d::extract_blender_thumbnail(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
