use crate::core::error::AppResult;
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for RAW photography formats using a tiered extraction strategy.
///
/// Strategies:
/// 1. LibRaw (rsraw) - Fast and official preview extraction.
/// 2. Brute-Force JPEG Scan - Search for JPEG markers in file headers.
/// 3. FFmpeg Fallback - Ultimate fallback for complex formats.
#[derive(Default)]
pub struct RawFormatProvider;

impl RawFormatProvider {
    /// Create a new instance of `RawFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Implement the FormatProvider trait for RawFormatProvider.
impl FormatProvider for RawFormatProvider {
    /// Return the name of the format provider.
    ///
    /// # Returns
    ///
    /// * `&'static str` - The name of the format provider.
    fn name(&self) -> &'static str {
        "RAW_PHOTOGRAPHY_PROVIDER"
    }

    /// Return the supported extensions for the format provider.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - The supported extensions for the format provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["arw", "cr2", "cr3", "dng", "nef", "nrw", "orf", "raf", "rw2", "pef", "srw", "x3f"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Sony RAW Image",
                vec!["arw"],
                vec!["image/x-sony-arw"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Canon RAW Image",
                vec!["cr2", "cr3"],
                vec!["image/x-canon-cr2", "image/x-canon-cr3"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Digital Negative",
                vec!["dng"],
                vec!["image/x-adobe-dng"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Nikon RAW Image",
                vec!["nef", "nrw"],
                vec!["image/x-nikon-nef", "image/x-nikon-nrw"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Olympus RAW Image",
                vec!["orf"],
                vec!["image/x-olympus-orf"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Fujifilm RAW Image",
                vec!["raf"],
                vec!["image/x-fujifilm-raf"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Panasonic RAW Image",
                vec!["rw2"],
                vec!["image/x-panasonic-raw"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Pentax RAW Image",
                vec!["pef"],
                vec!["image/x-pentax-pef"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Samsung RAW Image",
                vec!["srw"],
                vec!["image/x-samsung-srw"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Sigma RAW Image",
                vec!["x3f"],
                vec!["image/x-sigma-x3f"],
                MediaType::Image,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Return whether the format provider supports magic bytes.
    ///
    /// # Arguments
    ///
    /// * `_header_bytes` - The header bytes of the file.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the format provider supports magic bytes.
    fn supports_magic_bytes(&self, _header_bytes: &[u8]) -> bool {
        // Most RAW files are TIFF-based or have specific headers,
        // but since we rely on extensions first, we'll keep this light
        // or implement specific magic bytes if needed for extension-less files.
        false
    }

    /// Return the metadata capability of the format provider.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn MetadataCapability>` - The metadata capability of the format provider.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Return the thumbnail capability of the format provider.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn ThumbnailCapability>` - The thumbnail capability of the format provider.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implement the MetadataCapability trait for RawFormatProvider.
#[async_trait]
impl MetadataCapability for RawFormatProvider {
    /// Extract technical metadata from the file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// * `AppResult<serde_json::Value>` - The technical metadata of the file.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file =
                std::fs::File::open(&path_owned).map_err(crate::core::error::AppError::Io)?;
            let mmap = unsafe {
                memmap2::MmapOptions::new()
                    .map(&file)
                    .map_err(crate::core::error::AppError::Io)?
            };

            let raw = rsraw::RawImage::open(&mmap).map_err(|e| {
                crate::core::error::AppError::Generic(format!("LibRaw metadata error: {:?}", e))
            })?;

            // LibRaw might provide dimensions and some basic EXIF
            Ok(serde_json::json!({
                "width": raw.width(),
                "height": raw.height(),
                // In a real scenario, we'd extract more EXIF here if rsraw supports it
                // or use a dedicated EXIF crate.
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extract semantic metadata from the file.
    ///
    /// # Arguments
    ///
    /// * `_path` - The path to the file.
    ///
    /// # Returns
    ///
    /// * `AppResult<serde_json::Value>` - The semantic metadata of the file.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

/// Implement the ThumbnailCapability trait for RawFormatProvider.
#[async_trait]
impl ThumbnailCapability for RawFormatProvider {
    /// Generate a thumbnail for the file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - The thumbnail of the file.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            // Tier 1: LibRaw
            if let Ok(data) = extract_libraw_preview(&path_owned) {
                if let Ok(img) = image::load_from_memory(&data) {
                    return process_and_encode_webp(img, size_hint);
                }
            }

            // Tier 2: Brute Force JPEG Scan
            if let Ok(img) = brute_force_scan_jpeg(&path_owned) {
                return process_and_encode_webp(img, size_hint);
            }

            // Tier 3: FFmpeg Fallback (Not implemented here, but typically registry would fallback
            // or we could call our ffmpeg utility if integrated)

            Err(crate::core::error::AppError::Generic(format!(
                "Failed to extract RAW preview for {:?}",
                path_owned
            )))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

/// Helper: LibRaw preview extraction.
///
/// # Arguments
///
/// * `path` - The path to the file.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The preview of the file.
fn extract_libraw_preview(path: &Path) -> AppResult<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .map_err(crate::core::error::AppError::Io)?
    };

    let mut raw = rsraw::RawImage::open(&mmap).map_err(|e| {
        crate::core::error::AppError::Generic(format!("LibRaw open error: {:?}", e))
    })?;

    let thumbs = raw.extract_thumbs().map_err(|e| {
        crate::core::error::AppError::Generic(format!("LibRaw thumb error: {:?}", e))
    })?;

    thumbs
        .iter()
        .max_by_key(|t| t.width * t.height)
        .map(|t| t.data.clone())
        .ok_or_else(|| crate::core::error::AppError::Generic("No thumbnails found".into()))
}

/// Helper: Brute force JPEG scanner.
///
/// # Arguments
///
/// * `path` - The path to the file.
///
/// # Returns
///
/// * `AppResult<image::DynamicImage>` - The image of the file.
fn brute_force_scan_jpeg(path: &Path) -> AppResult<image::DynamicImage> {
    let file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .map_err(crate::core::error::AppError::Io)?
    };

    let mut best_img: Option<image::DynamicImage> = None;
    let mut best_size = 0;

    let scan_limit = mmap.len().min(8 * 1024 * 1024);
    let mut i = 0;
    while i < scan_limit - 4 {
        if mmap[i] == 0xFF && mmap[i + 1] == 0xD8 && mmap[i + 2] == 0xFF {
            if let Ok(img) = image::load_from_memory(&mmap[i..]) {
                let s = img.width() * img.height();
                if s > best_size {
                    best_size = s;
                    best_img = Some(img);
                }
                i += 2048;
                continue;
            }
        }
        i += 1;
    }

    best_img.ok_or_else(|| crate::core::error::AppError::Generic("Brute force failed".into()))
}

/// Helper: Process and encode to WebP.
///
/// # Arguments
///
/// * `img` - The image to process.
///
/// * `size_hint` - The size hint for the image.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The processed image.
pub(crate) fn process_and_encode_webp(
    img: image::DynamicImage,
    size_hint: u32,
) -> AppResult<Vec<u8>> {
    use fast_image_resize as fr;

    let width = img.width();
    let height = img.height();

    let aspect = width as f32 / height as f32;
    let (new_w, new_h) = if aspect > 1.0 {
        (size_hint, (size_hint as f32 / aspect).max(1.0) as u32)
    } else {
        (((size_hint as f32 * aspect).max(1.0)) as u32, size_hint)
    };

    let src_image = fr::images::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

    let mut dst_image = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);
    let mut resizer = fr::Resizer::new();
    resizer
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

    let encoder = webp::Encoder::from_rgba(dst_image.buffer(), new_w, new_h);
    let webp_data = encoder.encode(80.0);

    Ok(webp_data.to_vec())
}
