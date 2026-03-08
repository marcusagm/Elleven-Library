use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::path::Path;
use tiny_skia::Pixmap;

/// Provider for Font files (.ttf, .otf).
pub struct FontFormatProvider;

impl FontFormatProvider {
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for FontFormatProvider {
    fn name(&self) -> &'static str {
        "FONT_FORMAT_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ttf", "otf", "woff", "woff2"]
    }

    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // TTF/OTF magic bytes or common headers
        header_bytes.starts_with(&[0, 1, 0, 0, 0]) || // TTF
        header_bytes.starts_with(b"OTTO") ||         // OTF
        header_bytes.starts_with(b"wOFF") // WOFF
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for FontFormatProvider {
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let font_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let face = ttf_parser::Face::parse(&font_data, 0).map_err(|e| {
                crate::core::error::AppError::Generic(format!("Font parse error: {:?}", e))
            })?;

            let mut family_name = None;
            for name in face.names() {
                if name.name_id == ttf_parser::name_id::FULL_NAME {
                    family_name = name.to_string();
                    break;
                }
            }

            Ok(serde_json::json!({
                "family": family_name,
                "is_bold": face.is_regular(),
                "units_per_em": face.units_per_em(),
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for FontFormatProvider {
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let _data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;

            // Render a glyph (e.g., 'A') to a pixmap
            // This is a simplified version, in a real app we'd use a more robust
            // glyph renderer or a library like 'ab_glyph' for better results.

            let mut pixmap = Pixmap::new(size_hint, size_hint).ok_or_else(|| {
                crate::core::error::AppError::Generic("Failed to create pixmap".into())
            })?;

            // Fill with dark grey for placeholder font preview
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(30, 30, 30, 255);
            pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(0.0, 0.0, size_hint as f32, size_hint as f32).unwrap(),
                &paint,
                tiny_skia::Transform::identity(),
                None,
            );

            // Encode to WebP
            let encoder = webp::Encoder::from_rgba(pixmap.data(), size_hint, size_hint);
            let webp_data = encoder.encode(80.0);
            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
