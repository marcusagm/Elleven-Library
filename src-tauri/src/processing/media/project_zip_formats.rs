use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use crate::processing::media::extractors;
use async_trait::async_trait;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for ZIP-based design projects (Krita, Sketch, etc.)
#[derive(Default)]
pub struct ProjectZipFormatProvider;

/// Implementação do provedor de formato de imagem.
impl ProjectZipFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `ProjectZipFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }

    /// Tenta encontrar uma prévia em um contêiner tipo ZIP.
    fn extract_preview_from_zip(&self, path: &Path) -> AppResult<(Vec<u8>, String)> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Use specialized extractors if available (V1 Parity)
        match ext.as_str() {
            "penpot" => {
                return extractors::extract_penpot_preview(path)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            }
            _ => {}
        }

        let file = File::open(path).map_err(crate::core::error::AppError::Io)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

        // Map extension to likely preview path
        let preview_paths = match ext.as_str() {
            "fig" => vec!["preview.png", "thumbnail.png"],
            _ => vec!["preview.png", "thumbnail.png", "previews/preview.png"],
        };

        for p in preview_paths {
            if let Ok(mut zip_file) = archive.by_name(p) {
                let mut buffer = Vec::new();
                zip_file
                    .read_to_end(&mut buffer)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((buffer, "image/png".to_string()));
            }
        }

        Err(crate::core::error::AppError::FormatNotSupported(format!(
            "No preview found in ZIP project: {}",
            ext
        )))
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for ProjectZipFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "PROJECT_ZIP_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["fig", "penpot"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![

            SupportedFormat::with_metadata(
                "Figma Archive",
                vec!["fig"],
                vec!["application/x-figma"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),

            SupportedFormat::with_metadata(
                "Penpot Project",
                vec!["penpot"],
                vec!["application/x-penpot"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Verifica se o provedor suporta magic bytes específicos.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"PK\x03\x04")
            || header_bytes.starts_with(&[0x01, 0x0B, 0x1A, 0x86])
    }

    /// Retorna o provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Retorna o provedor de thumbnail.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Retorna o provedor de preview.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl ThumbnailCapability for ProjectZipFormatProvider {
    /// Gera uma thumbnail para o arquivo.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || -> AppResult<Vec<u8>> {
            let provider = ProjectZipFormatProvider::new();
            provider
                .extract_preview_from_zip(&path_owned)
                .map(|(d, _)| d)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

/// Implementação da capacidade de Preview.
#[async_trait]
impl PreviewCapability for ProjectZipFormatProvider {
    /// Gera um preview de alta resolução para o arquivo.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || -> AppResult<(Vec<u8>, String)> {
            let provider = ProjectZipFormatProvider::new();
            provider.extract_preview_from_zip(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

/// Implementação da capacidade de Metadados.
#[async_trait]
impl MetadataCapability for ProjectZipFormatProvider {
    async fn extract_technical(&self, path: &Path) -> AppResult<Value> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if extension == "penpot" {
            let path_owned = path.to_path_buf();
            let metadata = tokio::task::spawn_blocking(move || -> Option<Value> {
                let mut file = File::open(&path_owned).ok()?;
                let mut header = [0u8; 4];
                if file.read(&mut header).ok()? < 4 {
                    return None;
                }

                // Only ZIP V1 has manifest.json easily accessible for dimensions
                if header == [0x50, 0x4B, 0x03, 0x04] {
                    let mut archive = zip::ZipArchive::new(file).ok()?;
                    let mut manifest_entry = archive.by_name("manifest.json").ok()?;
                    let mut manifest_content = String::new();
                    manifest_entry.read_to_string(&mut manifest_content).ok()?;
                    let manifest_json: Value = serde_json::from_str(&manifest_content).ok()?;

                    let width = manifest_json["width"].as_f64().unwrap_or(0.0) as u32;
                    let height = manifest_json["height"].as_f64().unwrap_or(0.0) as u32;

                    return Some(serde_json::json!({
                        "container": "ZIP (Penpot V1)",
                        "width": width,
                        "height": height,
                        "metadata_source": "manifest.json"
                    }));
                }

                // V2 (Zstd)
                if header == [0x01, 0x0B, 0x1A, 0x86] {
                    return Some(serde_json::json!({
                        "container": "Zstd (Penpot V2)",
                        "metadata_support": "Thumbnail Only"
                    }));
                }

                None
            })
            .await
            .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?;

            if let Some(data) = metadata {
                return Ok(data);
            }
        }

        // Basic metadata fallback
        Ok(serde_json::json!({
            "container": "ZIP / Project Archive",
            "metadata_support": "Limited (V2)"
        }))
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}
