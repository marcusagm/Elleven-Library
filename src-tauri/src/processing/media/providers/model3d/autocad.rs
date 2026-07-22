use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for AutoCAD drawing files (.dwg, .dxf).
///
/// DWG and DXF are the industry-standard formats for 2D/3D CAD drawings.
/// Preview is generated via Assimp conversion to GLB.
///
/// # Technical Details
///
/// - **File Format**: DWG (binary), DXF (ASCII/binary)
/// - **Thumbnail**: Not yet implemented
/// - **Preview**: Conversion to GLB via Assimp
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::model3d::autocad::AutocadFormatProvider;
///
/// let provider = AutocadFormatProvider::new();
/// let extensions = provider.supported_extensions();
/// assert!(extensions.contains(&"dwg"));
/// assert!(extensions.contains(&"dxf"));
/// ```
#[derive(Default)]
pub struct AutocadFormatProvider;

impl AutocadFormatProvider {
    /// Creates a new instance of `AutocadFormatProvider`.
    ///
    /// # Returns
    ///
    /// `AutocadFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for AutocadFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "AUTOCAD_FORMAT_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["dwg", "dxf"]
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
            "AutoCAD Drawing",
            vec!["dwg", "dxf"],
            vec!["image/vnd.dwg", "image/vnd.dxf"],
            MediaType::Model3D,
            ThumbnailStrategy::None,
            PreviewStrategy::Assimp,
            PlaybackStrategy::None,
        )]
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
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
impl MetadataCapability for AutocadFormatProvider {
    /// Returns empty technical metadata (not yet implemented).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the AutoCAD file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    /// Returns empty semantic metadata (not yet implemented).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the AutoCAD file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for AutocadFormatProvider {
    /// Converts the AutoCAD file to GLB format via Assimp for browser-based preview.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the AutoCAD file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Transcoding` - If Assimp is not available or conversion fails.
    /// * `AppError::Io` - If file I/O operations fail.
    /// * `AppError::ExtractionProcessTimeout` - If the conversion times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        crate::processing::media::extractors::model3d::convert_to_glb_with_assimp(path, asset_id)
            .await
    }
}
