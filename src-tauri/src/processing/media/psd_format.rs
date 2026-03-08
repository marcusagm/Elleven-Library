use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Adobe Photoshop files (.psd, .psb).
pub struct PsdFormatProvider;

/// Implementação do provedor de formato de imagem.
impl PsdFormatProvider {
    /// Create a new instance of `PsdFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for PsdFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "ADOBE_PHOTOSHOP_PROVIDER"
    }

    /// Extensões de arquivos suportadas para PSD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["psd", "psb"]
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
        header_bytes.starts_with(b"8BPS")
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
impl MetadataCapability for PsdFormatProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados técnicos.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let psd = psd::Psd::from_bytes(&data)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            Ok(serde_json::json!({
                "width": psd.width(),
                "height": psd.height(),
                "color_mode": format!("{:?}", psd.color_mode()),
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
    /// `AppResult<serde_json::Value>` - Metadados semânticos.
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let psd = psd::Psd::from_bytes(&data)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            // Extract layer names if available
            let layer_names: Vec<String> =
                psd.layers().iter().map(|l| l.name().to_string()).collect();

            Ok(serde_json::json!({
                "layer_names": layer_names,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

/// Implementação da capacidade de thumbnail.
#[async_trait]
impl ThumbnailCapability for PsdFormatProvider {
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
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let psd = psd::Psd::from_bytes(&data)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            let rgba = psd.rgba();

            // Create image from RGBA pixels
            let rgba_buffer = image::RgbaImage::from_raw(psd.width(), psd.height(), rgba)
                .ok_or_else(|| {
                    crate::core::error::AppError::Generic(
                        "Failed to create image buffer from PSD pixels".into(),
                    )
                })?;
            let img = image::DynamicImage::ImageRgba8(rgba_buffer);

            // Use the shared helper from raw_format to resize and encode
            super::raw_format::process_and_encode_webp(img, size_hint)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
