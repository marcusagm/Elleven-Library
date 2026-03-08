use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for XMind files (.xmind)
pub struct XMindFormatProvider;

/// Implementação do provedor de formato de imagem.
impl XMindFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `XMindFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for XMindFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "XMIND_PROVIDER"
    }

    /// Extensões de arquivos suportadas para XMind.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["xmind"]
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
impl ThumbnailCapability for XMindFormatProvider {
    /// Gera uma thumbnail do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    /// `size_hint` - Hint de tamanho da thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail do arquivo.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            // XMind 8+: Thumbnails/thumbnail.png
            // XMind ZEN: preview.png
            let preview_paths = vec!["Thumbnails/thumbnail.png", "preview.png"];

            for p in preview_paths {
                if let Ok(mut zip_file) = archive.by_name(p) {
                    let mut buffer = Vec::new();
                    zip_file
                        .read_to_end(&mut buffer)
                        .map_err(crate::core::error::AppError::Io)?;
                    return Ok(buffer);
                }
            }

            Err(crate::core::error::AppError::FormatNotSupported(
                "No preview found in XMind file".into(),
            ))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
