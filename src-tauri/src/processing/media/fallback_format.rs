//! Provedor de fallback genérico.
//!
//! Atuando como a última linha de defesa, este provedor tenta identificar
//! arquivos via Magic Bytes (usando a crate `infer`) quando a resolução por
//! extensão falha. Não emite metadados, servindo apenas para evitar que
//! o arquivo seja totalmente ignorado pelo indexador.

use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

/// Implementação do provedor de fallback genérico.
pub struct GenericByteFallbackProvider {}

/// Implementação do provedor de fallback genérico.
impl Default for GenericByteFallbackProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericByteFallbackProvider {
    pub fn new() -> Self {
        Self {}
    }
}

/// Implementação do provedor de metadados.
#[async_trait]
impl FormatProvider for GenericByteFallbackProvider {
    /// Retorna o nome do provedor.
    ///
    /// # Returns
    ///
    /// * `&'static str` - O nome do provedor.
    fn name(&self) -> &'static str {
        "GENERIC_BYTE_FALLBACK"
    }

    /// Retorna as extensões suportadas.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - As extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        // Não possui extensões fixas, atua apenas no loop de fallback/deep check
        vec![]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Binary Fallback",
                vec!["bin"],
                vec!["application/octet-stream"],
                MediaType::Unknown,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Verifica se o provedor suporta o arquivo baseado em Magic Bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - Os primeiros bytes do arquivo.
    ///
    /// # Returns
    ///
    /// * `bool` - Indica se o provedor suporta o arquivo.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // Se conseguirmos identificar qualquer tipo via infer, aceitamos o "aperto de mão"
        // para dar algum contexto ao sistema, mesmo que não tenhamos capability.
        infer::get(header_bytes).is_some()
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn MetadataCapability>` - O provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    // Não fornece ThumbnailCapability
}

/// Implementação do provedor de metadados.
#[async_trait]
impl MetadataCapability for GenericByteFallbackProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// * `_path` - O caminho para o arquivo.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados técnicos.
    async fn extract_technical(&self, _path: &Path) -> AppResult<Value> {
        // Conforme solicitado na Sprint: apenas atira um erro visual para o Front
        Err(AppError::NoResolutionLimit)
    }

    /// Extrai metadados semânticos do arquivo.
    ///
    /// # Arguments
    ///
    /// * `_path` - O caminho para o arquivo.
    ///
    /// # Returns
    ///
    /// * `AppResult<Value>` - O resultado da extração de metadados semânticos.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}
