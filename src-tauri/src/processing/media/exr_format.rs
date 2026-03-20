use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for OpenEXR image files (.exr).
#[derive(Default)]
pub struct ExrFormatProvider;

/// Implementação do provedor de formato de imagem.
impl ExrFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `ExrFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for ExrFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "OPENEXR_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["exr"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};

        vec![
            SupportedFormat::with_metadata(
                "OpenEXR Image",
                vec!["exr"],
                vec!["image/x-exr"],
                MediaType::Image,
                ThumbnailStrategy::Raw,
                PreviewStrategy::Convert,
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
        // EXR magic bytes: 76 2F 31 01
        header_bytes.starts_with(b"\x76\x2F\x31\x01")
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
impl MetadataCapability for ExrFormatProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados técnicos do arquivo.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let img = image::open(&path_owned)
                .map_err(|e| crate::core::error::AppError::Generic(format!("EXR error: {}", e)))?;

            Ok(serde_json::json!({
                "width": img.width(),
                "height": img.height(),
                "color_type": format!("{:?}", img.color()),
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
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
impl ThumbnailCapability for ExrFormatProvider {
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
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let img = image::open(&path_owned)
                .map_err(|e| crate::core::error::AppError::Generic(format!("EXR error: {}", e)))?;

            // Use the shared helper from raw_format to resize and encode
            super::raw_format::process_and_encode_webp(img, size_hint)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
