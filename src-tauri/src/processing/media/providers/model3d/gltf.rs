use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for GL Transmission Format files (.gltf, .glb).
///
/// GLTF/GLB files are the standard interchange format for 3D models on the
/// web. GLB files can be previewed natively in the browser, while GLTF files
/// (JSON + binary references) are also browser-native.
///
/// # Technical Details
///
/// - **File Format**: glTF 2.0 (JSON or Binary)
/// - **Preview**: Browser-native (Three.js / model-viewer)
/// - **Thumbnail**: Not yet implemented
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::model3d::gltf::GltfFormatProvider;
///
/// let provider = GltfFormatProvider::new();
/// let extensions = provider.supported_extensions();
/// assert!(extensions.contains(&"gltf"));
/// assert!(extensions.contains(&"glb"));
/// ```
#[derive(Default)]
pub struct GltfFormatProvider;

impl GltfFormatProvider {
    /// Creates a new instance of `GltfFormatProvider`.
    ///
    /// # Returns
    ///
    /// `GltfFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for GltfFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "GLTF_FORMAT_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["gltf", "glb"]
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
            "GL Transmission Format",
            vec!["gltf", "glb"],
            vec!["model/gltf+json", "model/gltf-binary"],
            MediaType::Model3D,
            ThumbnailStrategy::None,
            PreviewStrategy::BrowserNative,
            PlaybackStrategy::None,
        )]
    }

    /// Validates glTF magic bytes (`glTF` for binary GLB).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header matches the GLB binary signature.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"glTF")
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
impl MetadataCapability for GltfFormatProvider {
    /// Returns empty technical metadata (not yet implemented for glTF).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the glTF file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    /// Returns empty semantic metadata (not yet implemented for glTF).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the glTF file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl PreviewCapability for GltfFormatProvider {
    /// Serves the raw GLB/glTF data for browser-native 3D rendering.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the glTF/GLB file.
    /// * `_asset_id` - The ID of the asset (unused).
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let extension = path_owned
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("glb")
            .to_lowercase();

        let data = tokio::fs::read(&path_owned)
            .await
            .map_err(crate::core::error::AppError::Io)?;
        let mime = if extension == "glb" {
            "model/gltf-binary"
        } else {
            "model/gltf+json"
        };
        Ok((data, mime.to_string()))
    }
}
