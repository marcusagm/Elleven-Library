use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Aseprite files (.ase, .aseprite).
pub struct AsepriteFormatProvider;

/// Implementação do provedor de formato de imagem.
impl AsepriteFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `AsepriteFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for AsepriteFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "ASEPRITE_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ase", "aseprite"]
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
        // Aseprite files start with the file size (4 bytes) followed by magic 0xA5E0 (2 bytes) at offset 4
        if header_bytes.len() < 6 {
            return false;
        }
        header_bytes[4] == 0xE0 && header_bytes[5] == 0xA5
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
impl MetadataCapability for AsepriteFormatProvider {
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
            let ase = asefile::AsepriteFile::read_file(&path_owned)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            Ok(serde_json::json!({
                "width": ase.width(),
                "height": ase.height(),
                "num_frames": ase.num_frames(),
                "num_layers": ase.num_layers(),
                "color_depth": format!("{:?}", ase.pixel_format()),
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
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let ase = asefile::AsepriteFile::read_file(&path_owned)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            let layer_names: Vec<String> = (0..ase.num_layers())
                .map(|i| ase.layer(i).name().to_string())
                .collect();

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
impl ThumbnailCapability for AsepriteFormatProvider {
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
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let ase = asefile::AsepriteFile::read_file(&path_owned)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            // Generate image from the first frame
            let frame_image = ase.frame(0).image();
            let (width, height) = (frame_image.width(), frame_image.height());
            let raw_pixels = frame_image.into_raw();

            // Create image 0.25 DynamicImage from raw pixels
            let rgba_buffer =
                image::RgbaImage::from_raw(width, height, raw_pixels).ok_or_else(|| {
                    crate::core::error::AppError::Generic(
                        "Failed to create RGBA buffer from Aseprite pixels".into(),
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
