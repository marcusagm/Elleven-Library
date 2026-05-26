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

fn extract_largest_icns_image(icns_bytes: &[u8]) -> Option<(Vec<u8>, usize, usize)> {
    if icns_bytes.len() < 8 || &icns_bytes[0..4] != b"icns" {
        return None;
    }

    let file_length = u32::from_be_bytes(icns_bytes[4..8].try_into().ok()?) as usize;
    let actual_length = icns_bytes.len().min(file_length);

    let mut scan_offset = 8usize;
    let mut best_image: Option<(Vec<u8>, usize, usize)> = None;
    let mut best_area = 0usize;

    while scan_offset + 8 <= actual_length {
        let block_length = u32::from_be_bytes(
            icns_bytes[scan_offset + 4..scan_offset + 8]
                .try_into()
                .ok()?,
        ) as usize;

        if block_length < 8 || scan_offset + block_length > actual_length {
            break;
        }

        let block_data = &icns_bytes[scan_offset + 8..scan_offset + block_length];

        let is_png = block_data.starts_with(b"\x89PNG\r\n\x1a\n");
        let is_jpeg = block_data.starts_with(b"\xFF\xD8\xFF");

        if is_png || is_jpeg {
            if let Ok(dimensions) = imagesize::blob_size(block_data) {
                let area = dimensions.width * dimensions.height;
                if area > best_area {
                    best_area = area;
                    best_image = Some((block_data.to_vec(), dimensions.width, dimensions.height));
                }
            }
        }

        scan_offset += block_length;
    }

    best_image
}

#[async_trait]
impl MetadataCapability for IcnsFormatProvider {
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let icns_bytes = std::fs::read(&path_owned).map_err(AppError::Io)?;
            if let Some((_, width, height)) = extract_largest_icns_image(&icns_bytes) {
                Ok(serde_json::json!({
                    "width": width,
                    "height": height,
                    "format": "ICNS",
                }))
            } else {
                Err(AppError::Generic(
                    "Failed to extract valid icon image from ICNS".into(),
                ))
            }
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
            let icns_bytes = std::fs::read(&path_owned).map_err(AppError::Io)?;
            if let Some((image_bytes, _, _)) = extract_largest_icns_image(&icns_bytes) {
                let decoded_image =
                    image::load_from_memory(&image_bytes).map_err(|decode_error| {
                        AppError::Generic(format!("Failed to decode ICNS image: {}", decode_error))
                    })?;
                crate::processing::media::extractors::image::process_and_encode_webp(
                    decoded_image,
                    size_hint,
                )
            } else {
                Err(AppError::Generic(
                    "Failed to extract valid icon image from ICNS".into(),
                ))
            }
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
            let icns_bytes = std::fs::read(&path_owned).map_err(AppError::Io)?;
            if let Some((image_bytes, _, _)) = extract_largest_icns_image(&icns_bytes) {
                let decoded_image =
                    image::load_from_memory(&image_bytes).map_err(|decode_error| {
                        AppError::Generic(format!("Failed to decode ICNS image: {}", decode_error))
                    })?;
                let webp_bytes =
                    crate::processing::media::extractors::image::process_and_encode_webp(
                        decoded_image,
                        2048,
                    )?;
                Ok((webp_bytes, "image/webp".to_string()))
            } else {
                Err(AppError::Generic(
                    "Failed to extract valid icon image from ICNS".into(),
                ))
            }
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_magic_bytes() {
        let provider = IcnsFormatProvider::new();
        assert!(provider.supports_magic_bytes(b"icns\x00\x00\x00\x08"));
        assert!(!provider.supports_magic_bytes(b"png_\x00\x00\x00\x08"));
    }

    #[test]
    fn test_extract_largest_icns_image_invalid() {
        assert!(extract_largest_icns_image(b"").is_none());
        assert!(extract_largest_icns_image(b"icns").is_none());
        assert!(extract_largest_icns_image(b"icns\x00\x00\x00\x10ic08\x00\x00\x00\x00").is_none());
    }
}
