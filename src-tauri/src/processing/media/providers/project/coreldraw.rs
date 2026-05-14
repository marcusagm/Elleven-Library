//! CorelDRAW (.cdr) format provider.
//!
//! Provides metadata, thumbnail, and preview extraction for all CorelDRAW
//! versions (v3 to modern ZIP-based v24+).

use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::formats::types::{
    MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
};
use crate::core::AppResult;
use crate::processing::media::extractors;
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;
use tracing::instrument;

/// Provider for CorelDRAW (.cdr) files.
#[derive(Default)]
pub struct CoreldrawFormatProvider;

impl CoreldrawFormatProvider {
    /// Creates a new instance of the CorelDRAW format provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for CoreldrawFormatProvider {
    fn name(&self) -> &'static str {
        "CORELDRAW_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["cdr"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![SupportedFormat::with_metadata(
            "CorelDRAW Drawing",
            vec!["cdr"],
            vec!["application/x-coreldraw", "application/vnd.corel-draw"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // Modern ZIP: PK..
        header_bytes.starts_with(b"PK\x03\x04") ||
        // Legacy RIFF: RIFF....CDR
        (header_bytes.starts_with(b"RIFF") && header_bytes.len() >= 12 && 
         (&header_bytes[8..11] == b"CDR" || &header_bytes[8..11] == b"cdr")) ||
        // Ancient WL: WL
        header_bytes.starts_with(b"WL")
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl ThumbnailCapability for CoreldrawFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_coreldraw_preview(&path_owned)
                .map(|(data, _)| data)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for CoreldrawFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            // Use highres extraction for the preview modal (upscaled with Lanczos3 if needed)
            extractors::extract_coreldraw_preview_highres(&path_owned, 2048)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl MetadataCapability for CoreldrawFormatProvider {
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_coreldraw_metadata(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<Value> {
        Ok(serde_json::json!({}))
    }
}
