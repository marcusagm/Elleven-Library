use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use fast_image_resize as fr;
use std::path::Path;
use zune_jpeg::JpegDecoder;

/// Provider for rasterized image formats (JPG, PNG, WebP, etc.)
pub struct ImageFormatProvider;

/// Implementation of `ImageFormatProvider`.
impl ImageFormatProvider {
    /// Create a new instance of `ImageFormatProvider`.
    ///
    /// # Returns
    ///
    /// A new instance of `ImageFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for ImageFormatProvider {
    /// Get the name of the format provider.
    ///
    /// # Returns
    ///
    /// A `&'static str` containing the name of the format provider.
    fn name(&self) -> &'static str {
        "RASTER_IMAGE_PROVIDER"
    }

    /// Get the supported extensions for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec![
            "jpg", "jpeg", "jpe", "jfif", "png", "webp", "gif", "bmp", "ico", "tiff", "tif",
        ]
    }

    /// Check if the given header bytes support the format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes of the file.
    ///
    /// # Returns
    ///
    /// `true` if the format is supported, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // Basic check for common image formats
        header_bytes.starts_with(b"\xFF\xD8\xFF") || // JPEG
        header_bytes.starts_with(b"\x89PNG\r\n\x1a\n") || // PNG
        header_bytes.starts_with(b"RIFF") || // WebP (RIFF....WEBP)
        header_bytes.starts_with(b"GIF8") || // GIF
        header_bytes.starts_with(b"BM") // BMP
    }

    /// Trait for metadata extraction.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Trait for thumbnail generation.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Trait for metadata extraction.
#[async_trait]
impl MetadataCapability for ImageFormatProvider {
    /// Extract technical metadata from the given image.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the image file.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the technical metadata.
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let reader = image::ImageReader::open(&path_owned)
                .map_err(crate::core::error::AppError::Io)?
                .with_guessed_format()
                .map_err(crate::core::error::AppError::Io)?;

            let format = reader.format();
            let dimensions = reader
                .into_dimensions()
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            Ok(serde_json::json!({
                "width": dimensions.0,
                "height": dimensions.1,
                "format": format.map(|f| format!("{:?}", f)).unwrap_or_default(),
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extract semantic metadata from the given image.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the image file.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the semantic metadata.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        // Placeholder for future NLP/OCR extraction
        Ok(serde_json::json!({}))
    }
}

/// Trait for thumbnail generation.
#[async_trait]
impl ThumbnailCapability for ImageFormatProvider {
    /// Generate a thumbnail for the given image.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the image file.
    /// * `size_hint` - The desired size of the thumbnail.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the thumbnail data.
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let ext = path_owned
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // 1. Decode
            let (rgba_data, width, height) = match ext.as_str() {
                "jpg" | "jpeg" | "jpe" | "jfif" => {
                    let jpeg_data =
                        std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;
                    let mut decoder = JpegDecoder::new(&jpeg_data);
                    let pixels = decoder
                        .decode()
                        .map_err(|e| crate::core::error::AppError::Generic(format!("{:?}", e)))?;
                    let info = decoder.info().ok_or_else(|| {
                        crate::core::error::AppError::Generic("Failed to get JPEG info".to_string())
                    })?;

                    // Convert RGB to RGBA
                    let mut rgba = Vec::with_capacity(pixels.len() / 3 * 4);
                    for chunk in pixels.chunks_exact(3) {
                        rgba.push(chunk[0]);
                        rgba.push(chunk[1]);
                        rgba.push(chunk[2]);
                        rgba.push(255);
                    }
                    (rgba, info.width as u32, info.height as u32)
                }
                _ => {
                    let img = image::open(&path_owned)
                        .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;
                    let w = img.width();
                    let h = img.height();
                    (img.to_rgba8().into_raw(), w, h)
                }
            };

            // 2. Resize
            let aspect = width as f32 / height as f32;
            let (new_w, new_h) = if aspect > 1.0 {
                (size_hint, (size_hint as f32 / aspect).max(1.0) as u32)
            } else {
                (((size_hint as f32 * aspect).max(1.0)) as u32, size_hint)
            };

            let src_image =
                fr::images::Image::from_vec_u8(width, height, rgba_data, fr::PixelType::U8x4)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;
            let mut dst_image = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);
            let mut resizer = fr::Resizer::new();
            let options = fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear));

            resizer
                .resize(&src_image, &mut dst_image, Some(&options))
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            // 3. Encode to WebP
            let encoder = webp::Encoder::from_rgba(dst_image.buffer(), new_w, new_h);
            let webp_data = encoder.encode(80.0);

            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
