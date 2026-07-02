use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for Apple Icon Image files (.icns).
///
/// Parses the ICNS structure to extract the largest high-resolution PNG or JPEG
/// embedded sub-block, decoding and resizing it to generate thumbnails and previews.
#[derive(Default)]
pub struct IcnsFormatProvider;

impl IcnsFormatProvider {
    /// Creates a new instance of `IcnsFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for IcnsFormatProvider {
    fn name(&self) -> &'static str {
        "ICNS_IMAGE_PROVIDER"
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["icns"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "Apple Icon Image",
            vec!["icns"],
            vec!["image/x-icns"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::Convert,
            PlaybackStrategy::None,
        )]
    }

    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"icns")
    }

    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }

    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for IcnsFormatProvider {
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path_owned).map_err(AppError::Io)?;
            let file = std::io::BufReader::new(file);
            let icon_family = icns::IconFamily::read(file)
                .map_err(|e| AppError::Generic(format!("ICNS decode error: {}", e)))?;

            let mut max_width = 0;
            let mut max_height = 0;
            for icon in icon_family.available_icons() {
                let width = icon.pixel_width();
                let height = icon.pixel_height();
                if width > max_width {
                    max_width = width;
                }
                if height > max_height {
                    max_height = height;
                }
            }

            if max_width == 0 {
                return Err(AppError::Generic("No valid icons in ICNS".into()));
            }

            Ok(serde_json::json!({
                "width": max_width,
                "height": max_height,
                "format": "ICNS",
            }))
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for IcnsFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path_owned).map_err(AppError::Io)?;
            let file = std::io::BufReader::new(file);
            let icon_family = icns::IconFamily::read(file)
                .map_err(|e| AppError::Generic(format!("ICNS decode error: {}", e)))?;

            let mut best_icon = None;
            let mut max_width = 0;

            for icon in icon_family.available_icons() {
                let width = icon.pixel_width();
                if width >= size_hint && (best_icon.is_none() || width < max_width) {
                    best_icon = Some(icon);
                    max_width = width;
                }
            }

            if best_icon.is_none() {
                let mut largest = None;
                let mut l_width = 0;
                for icon in icon_family.available_icons() {
                    let width = icon.pixel_width();
                    if width > l_width {
                        largest = Some(icon);
                        l_width = width;
                    }
                }
                best_icon = largest;
            }

            let target_icon_type = best_icon
                .ok_or_else(|| AppError::Generic("No valid icons found in ICNS".into()))?;
            let image = icon_family
                .get_icon_with_type(target_icon_type)
                .map_err(|e| AppError::Generic(format!("Failed to extract icon type: {}", e)))?;

            let mut png_data = Vec::new();
            image.write_png(&mut png_data).map_err(|e| {
                AppError::Generic(format!("Failed to write ICNS to PNG buffer: {}", e))
            })?;

            let decoded_image = image::load_from_memory(&png_data)
                .map_err(|e| AppError::Generic(format!("Failed to decode ICNS PNG: {}", e)))?;

            crate::processing::media::extractors::image::process_and_encode_webp(
                decoded_image,
                size_hint,
            )
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for IcnsFormatProvider {
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path_owned).map_err(AppError::Io)?;
            let file = std::io::BufReader::new(file);
            let icon_family = icns::IconFamily::read(file)
                .map_err(|e| AppError::Generic(format!("ICNS decode error: {}", e)))?;

            let mut best_icon = None;
            let mut max_width = 0;
            for icon in icon_family.available_icons() {
                let width = icon.pixel_width();
                if width > max_width {
                    best_icon = Some(icon);
                    max_width = width;
                }
            }

            let target_icon_type = best_icon
                .ok_or_else(|| AppError::Generic("No valid icons found in ICNS".into()))?;
            let image = icon_family
                .get_icon_with_type(target_icon_type)
                .map_err(|e| AppError::Generic(format!("Failed to extract icon type: {}", e)))?;

            let mut png_data = Vec::new();
            image.write_png(&mut png_data).map_err(|e| {
                AppError::Generic(format!("Failed to write ICNS to PNG buffer: {}", e))
            })?;

            let decoded_image = image::load_from_memory(&png_data)
                .map_err(|e| AppError::Generic(format!("Failed to decode ICNS PNG: {}", e)))?;

            let webp_bytes = crate::processing::media::extractors::image::process_and_encode_webp(
                decoded_image,
                2048,
            )?;
            Ok((webp_bytes, "image/webp".to_string()))
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }
}
