use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for ZIP-based design projects (Krita, Sketch, etc.)
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
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    /// `extensions` - Extensões de arquivos suportadas.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Prévia do arquivo.
    fn extract_preview_from_zip(&self, path: &Path, _extensions: &[&str]) -> AppResult<Vec<u8>> {
        let file = File::open(path).map_err(crate::core::error::AppError::Io)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Map extension to likely preview path
        let preview_paths = match ext.as_str() {
            "kra" => vec!["preview.png"],
            "sketch" => vec!["previews/preview.png"],
            "mdp" => vec!["thumbnail.png", "preview.png"],
            "fig" => vec!["preview.png", "thumbnail.png"],
            "reb" => vec!["preview.png"],
            "xmind" => vec!["Thumbnails/thumbnail.png"],
            "cdr" => vec![
                "previews/preview.png",
                "metadata/thumbnails/thumbnail_v1.png",
            ],
            _ => vec!["preview.png", "thumbnail.png", "previews/preview.png"],
        };

        for p in preview_paths {
            if let Ok(mut zip_file) = archive.by_name(p) {
                let mut buffer = Vec::new();
                zip_file
                    .read_to_end(&mut buffer)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok(buffer);
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

    /// Verifica se o provedor suporta magic bytes específicos.
    ///
    /// # Arguments
    ///
    /// `header_bytes` - Bytes do cabeçalho do arquivo.
    ///
    /// # Returns
    ///
    /// `bool` - True se o provedor suporta os magic bytes, false caso contrário.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"PK\x03\x04")
    }

    /// Retorna o provedor de thumbnail.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - Provedor de thumbnail.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementação da capacidade de thumbnail.
#[async_trait]
impl ThumbnailCapability for ProjectZipFormatProvider {
    /// Gera uma thumbnail para o arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    /// `size_hint` - Hint de tamanho para a thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail do arquivo.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        let extensions = self.supported_extensions();

        tokio::task::spawn_blocking(move || {
            let provider = ProjectZipFormatProvider::new();
            provider.extract_preview_from_zip(&path_owned, &extensions)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
