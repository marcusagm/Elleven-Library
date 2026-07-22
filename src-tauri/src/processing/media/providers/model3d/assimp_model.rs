use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Assimp-compatible 3D model formats (.fbx, .obj, .dae, .stl, .3ds, .3mf, .lwo, .lws).
///
/// These formats require conversion to GLB via Assimp for browser-based
/// preview. Thumbnail generation is not yet implemented.
///
/// # Technical Details
///
/// - **File Format**: FBX, OBJ, Collada, STL, 3DS, 3MF, LightWave
/// - **Preview**: Conversion to GLB via Assimp
/// - **Thumbnail**: Not yet implemented
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::model3d::assimp_model::AssimpModelProvider;
///
/// let provider = AssimpModelProvider::new();
/// let extensions = provider.supported_extensions();
/// assert!(extensions.contains(&"fbx"));
/// assert!(extensions.contains(&"obj"));
/// ```
#[derive(Default)]
pub struct AssimpModelProvider;

impl AssimpModelProvider {
    /// Creates a new instance of `AssimpModelProvider`.
    ///
    /// # Returns
    ///
    /// `AssimpModelProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for AssimpModelProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "ASSIMP_3D_MODEL_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["fbx", "obj", "dae", "stl", "3ds", "3mf", "lwo", "lws"]
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
                "Autodesk FBX",
                vec!["fbx"],
                vec!["application/x-fbx"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Wavefront OBJ",
                vec!["obj"],
                vec!["model/obj"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Collada Model",
                vec!["dae"],
                vec!["model/vnd.collada+xml"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Stereolithography",
                vec!["stl"],
                vec!["model/stl"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "3D Studio Model",
                vec!["3ds"],
                vec!["application/x-3ds"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "3D Manufacturing Format",
                vec!["3mf"],
                vec!["model/3mf"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
        ]
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
impl MetadataCapability for AssimpModelProvider {
    /// Returns empty technical metadata (not yet implemented).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the 3D model file (unused).
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
    /// * `_path` - Path to the 3D model file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for AssimpModelProvider {
    /// Converts the 3D model to GLB format via Assimp for browser-based preview.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the 3D model file.
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
