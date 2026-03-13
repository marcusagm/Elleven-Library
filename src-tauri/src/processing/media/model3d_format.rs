use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability, PreviewCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tracing::{instrument, error};

/// Provider for 3D model formats (Blender, OBJ, GLTF, FBX, etc.)
#[derive(Default)]
pub struct Model3dFormatProvider;

impl Model3dFormatProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FormatProvider for Model3dFormatProvider {
    fn name(&self) -> &'static str {
        "3D_MODEL_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec![
            "blend", "fbx", "obj", "gltf", "glb", "dae", "stl", "3ds", "3mf", "dxf", "lwo", "lws",
        ]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Blender Project",
                vec!["blend"],
                vec!["application/x-blender"],
                MediaType::Model3D,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Autodesk FBX",
                vec!["fbx"],
                vec!["application/x-fbx"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Wavefront OBJ",
                vec!["obj"],
                vec!["model/obj"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "GL Transmission Format",
                vec!["gltf", "glb"],
                vec!["model/gltf+json", "model/gltf-binary"],
                MediaType::Model3D,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Collada Model",
                vec!["dae"],
                vec!["model/vnd.collada+xml"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Stereolithography",
                vec!["stl"],
                vec!["model/stl"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "3D Studio Model",
                vec!["3ds"],
                vec!["application/x-3ds"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "3D Manufacturing Format",
                vec!["3mf"],
                vec!["model/3mf"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "AutoCAD DXF",
                vec!["dxf"],
                vec!["image/vnd.dxf"],
                MediaType::Model3D,
                PreviewStrategy::Assimp,
                PlaybackStrategy::None,
            ),
        ]
    }

    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"BLENDER") || 
        header_bytes.starts_with(b"glTF") ||   
        header_bytes.starts_with(b"{") ||      
        header_bytes.starts_with(b"Kayak") 
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for Model3dFormatProvider {
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for Model3dFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        let ext = path_owned
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Handle Blender thumbnails (Native REND block)
        if ext == "blend" {
            let thumb = tokio::task::spawn_blocking(move || {
                let data = std::fs::read(&path_owned).map_err(AppError::Io)?;
                if let Some(pos) = data.windows(3).position(|w| w == b"\xFF\xD8\xFF") {
                    if let Some(end) = data[pos..].windows(2).position(|w| w == b"\xFF\xD9") {
                        return Ok(data[pos..pos + end + 2].to_vec());
                    }
                }
                Err(AppError::FormatNotSupported("No thumbnail in Blender file".into()))
            })
            .await
            .map_err(|_| AppError::ExtractionProcessTimeout)??;
            return Ok(thumb);
        }

        // 2. Fallback to a generic 3D icon or empty result
        // TODO: Could use assimp to get a metadata thumbnail if available
        Err(AppError::FormatNotSupported("3D thumbnail generation pending".into()))
    }
}

#[async_trait]
impl PreviewCapability for Model3dFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        let asset_id_owned = asset_id.to_string();
        let ext = path_owned
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Already GLB/GLTF, no need for conversion
        if ext == "glb" || ext == "gltf" {
            let data = tokio::fs::read(&path_owned).await.map_err(AppError::Io)?;
            let mime = if ext == "glb" { "model/gltf-binary" } else { "model/gltf+json" };
            return Ok((data, mime.to_string()));
        }

        let tools = crate::processing::transcoding::resolve_transcoding_tools::<tauri::Wry>(None)?;
        let assimp_bin = tools.assimp.ok_or_else(|| AppError::Transcoding("Assimp binary not found".to_string()))?;
        
        // Use a temporary directory for conversion
        let temp_dir = std::env::temp_dir().join(format!("mundam_3d_{}", asset_id_owned));
        tokio::fs::create_dir_all(&temp_dir).await.map_err(AppError::Io)?;
        
        let output_glb = temp_dir.join(format!("{}.glb", asset_id_owned));

        // Call Assimp
        let mut cmd = Command::new(assimp_bin);
        cmd.arg("export")
           .arg(&path_owned)
           .arg(&output_glb)
           .arg("-fglb2"); // Export as GLB v2

        let output = tokio::task::spawn_blocking(move || {
            cmd.output().map_err(AppError::Io)
        }).await.map_err(|_| AppError::ExtractionProcessTimeout)??;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Assimp conversion failed for {}: {}", asset_id_owned, err);
            return Err(AppError::Transcoding(format!("Assimp failed: {}", err)));
        }

        // Read the generated file
        let glb_data = tokio::fs::read(&output_glb).await.map_err(AppError::Io)?;
        
        // Cleanup temp file
        let _ = tokio::fs::remove_file(&output_glb).await;
        // ignore errors on removing temp dir
        let _ = tokio::fs::remove_dir(&temp_dir).await;

        Ok((glb_data, "model/gltf-binary".to_string()))
    }
}
