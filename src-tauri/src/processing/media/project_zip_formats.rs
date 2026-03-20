use crate::core::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, PreviewCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
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
            "mdp" => {
                return extractors::extract_mdp_preview(path)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            }
            "cdr" => {
                return extractors::extract_coreldraw_preview(path)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            }
            "sketch" => {
                return extractors::extract_sketch_preview(path)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            }
            "reb" => {
                return extractors::extract_rebelle_preview(path)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            }
            _ => {}
        }

        let file = File::open(path).map_err(crate::core::error::AppError::Io)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

        // V1 Parity logic for Krita: mergedimage.png is the high-res render
        if ext == "kra" {
            if let Ok(mut zip_file) = archive.by_name("mergedimage.png") {
                let mut buffer = Vec::new();
                zip_file
                    .read_to_end(&mut buffer)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((buffer, "image/png".to_string()));
            }
        }

        // Map extension to likely preview path
        let preview_paths = match ext.as_str() {
            "kra" => vec!["preview.png"],
            "fig" => vec!["preview.png", "thumbnail.png"],
            "xmind" => vec!["Thumbnails/thumbnail.png"],
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
        vec!["kra", "sketch", "mdp", "fig", "reb", "xmind", "cdr"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Krita Artwork",
                vec!["kra"],
                vec!["application/x-krita"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Sketch Project",
                vec!["sketch"],
                vec!["application/x-sketch"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "MediBang Paint / FireAlpaca",
                vec!["mdp"],
                vec!["application/x-medibang"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
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
                "Rebelle Artwork",
                vec!["reb"],
                vec!["application/x-rebelle"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "XMind Mindmap",
                vec!["xmind"],
                vec!["application/x-xmind"],
                MediaType::Project,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "CorelDRAW Drawing",
                vec!["cdr"],
                vec!["application/x-coreldraw"],
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
            provider.extract_preview_from_zip(&path_owned).map(|(d, _)| d)
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
    async fn extract_technical(&self, _path: &Path) -> AppResult<Value> {
        // Basic metadata to avoid errors. Can be expanded if we have a zip/project-specific metadata logic.
        Ok(serde_json::json!({
            "container": "ZIP",
            "metadata_support": "Limited (V2)"
        }))
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}
