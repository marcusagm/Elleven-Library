use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors::*;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for proprietary binary design formats (SAI, GIMP XCF, Corel Painter RIF, CLIP Studio)
#[derive(Default)]
pub struct BinaryDesignFormatProvider;

/// Implementação do provedor de formato de imagem.
impl BinaryDesignFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `BinaryDesignFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for BinaryDesignFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "BINARY_DESIGN_PROVIDER"
    }

    /// Extensões de arquivos suportadas para CAD.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["sai", "sai2", "xcf", "rif", "riff", "clip"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "PaintTool SAI v1",
                vec!["sai"],
                vec!["application/x-sai"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "PaintTool SAI v2",
                vec!["sai2"],
                vec!["application/x-sai2"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "GIMP Image",
                vec!["xcf"],
                vec!["image/x-xcf"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Corel Painter Image",
                vec!["rif", "riff"],
                vec!["application/x-painter"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Clip Studio Paint",
                vec!["clip"],
                vec!["application/x-clipstudio"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
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
        header_bytes.starts_with(b"gimp xcf") || // XCF
        header_bytes.starts_with(b"RIFF") ||     // RIFF (Painter)
        header_bytes.starts_with(b"SAI") ||      // SAI
        header_bytes.starts_with(b"CSFCHUNK")    // CLIP
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - Provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Retorna o provedor de thumbnails.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - O provedor de thumbnails.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementação da capacidade de metadados.
#[async_trait]
impl MetadataCapability for BinaryDesignFormatProvider {
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

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

/// Implementação do provedor de thumbnails.
#[async_trait]
impl ThumbnailCapability for BinaryDesignFormatProvider {
    /// Gera um thumbnail para o arquivo.
    ///
    /// # Arguments
    ///
    /// * `path` - O caminho para o arquivo.
    /// * `_size_hint` - O tamanho desejado para o thumbnail (não utilizado aqui
    ///                 pois estes formatos extraem previews embutidos).
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - O resultado da geração do thumbnail.
    async fn generate(&self, path: &Path, _size_hint: u32) -> crate::core::error::AppResult<Vec<u8>> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let result = match extension.as_str() {
            "sai" => extract_sai_preview(path),
            "sai2" => extract_sai2_preview(path),
            "xcf" => extract_xcf_preview(path),
            "clip" => extract_clip_preview(path),
            "rif" | "riff" => extract_corel_painter_preview(path),
            _ => Err("Unsupported extension for binary design provider".into()),
        };

        result
            .map(|(data, _mime)| data)
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
    }
}
