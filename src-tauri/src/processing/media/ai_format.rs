use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Adobe Illustrator files (.ai).
pub struct AiFormatProvider;

/// Implementação do provedor de formato de imagem.
impl AiFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `AiFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for AiFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "ADOBE_ILLUSTRATOR_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ai"]
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
        // Modern AI is PDF-based, Legacy AI is PostScript-based
        header_bytes.starts_with(b"%PDF-") || header_bytes.starts_with(b"%!PS-Adobe")
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - Provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
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

/// Implementação da capacidade de metadados.
#[async_trait]
impl MetadataCapability for AiFormatProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados técnicos do arquivo.
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({
            "format": "Adobe Illustrator"
        }))
    }

    /// Extrai metadados semânticos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados semânticos do arquivo.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

/// Implementação da capacidade de thumbnail.
#[async_trait]
impl ThumbnailCapability for AiFormatProvider {
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
        let _path_owned = path.to_path_buf();

        // Strategy:
        // 1. If PDF-based, we could use a PDF renderer (not yet implemented in V2).
        // 2. For now, we return error or icon fallback if no renderer is available.
        // In legacy, it might have called a native macOS extractor.

        Err(crate::core::error::AppError::FormatNotSupported(
            "AI thumbnail generation requires a PDF/PostScript renderer".to_string(),
        ))
    }
}
