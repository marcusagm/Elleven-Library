use crate::core::error::AppResult;
use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for CAD formats (STEP, IGES).
#[derive(Default)]
pub struct CadFormatProvider;

/// Implementação do provedor de formato de imagem.
impl CadFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `CadFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for CadFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "CAD_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["step", "stp", "iges", "igs"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "STEP Model",
                vec!["step", "stp"],
                vec!["application/step"],
                MediaType::Model3D,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "IGES Model",
                vec!["iges", "igs"],
                vec!["application/iges"],
                MediaType::Model3D,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
        ]
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
        header_bytes.starts_with(b"ISO-10303-21") || // STEP
        header_bytes.starts_with(b"S      1") ||       // IGES
        header_bytes.starts_with(b"      ") // IGES often has leading whitespace
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }
}

/// Implementação da capacidade de metadados.
#[async_trait]
impl MetadataCapability for CadFormatProvider {
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
        Ok(serde_json::json!({}))
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
