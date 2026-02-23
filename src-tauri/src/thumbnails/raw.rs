use std::path::Path;

/// Generates a thumbnail for a RAW image using rsraw (LibRaw) with brute-force fallbacks.
///
/// This leverages LibRaw's robust decoding for standard formats (CR3, RAF, ARW)
/// and falls back to a resilient binary scanner for rare or problematic files.
pub fn generate_raw_thumbnail(
    input_path: &Path,
    output_path: &Path,
    size_px: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Try robust LibRaw extraction first (standard market formats)
    if let Ok(data) = extract_raw_preview_data(input_path) {
        if let Ok(img) = image::load_from_memory(&data) {
            return process_image(img, output_path, size_px);
        }
    }

    // 2. Fallback: Efficient Brute Force JPEG Scan (works for rare/legacy formats)
    if let Ok(img) = brute_force_scan_jpeg(input_path) {
        return process_image(img, output_path, size_px);
    }

    // 3. Last Resort: Broad Binary Extractor (handles PNG/TIFF/XMP)
    if let Ok((data, _)) =
        crate::thumbnails::extractors::binary_jpeg::extract_any_embedded(input_path)
    {
        if let Ok(img) = image::load_from_memory(&data) {
            return process_image(img, output_path, size_px);
        }
    }

    Err(format!(
        "Failed all RAW preview extraction methods for {:?}",
        input_path
    )
    .into())
}

/// Scans the file for JPEG SOI markers and attempts to decode them.
/// This is very resilient to weird offsets and truncated files.
pub(crate) fn brute_force_scan_jpeg(
    path: &Path,
) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };

    let mut best_img: Option<image::DynamicImage> = None;
    let mut best_size = 0;

    // Scan first 8MB which usually contains all previews for most manufacturers.
    let scan_limit = mmap.len().min(8 * 1024 * 1024);
    let mut i = 0;
    while i < scan_limit - 4 {
        // JPEG SOI: FF D8 FF
        if mmap[i] == 0xFF && mmap[i + 1] == 0xD8 && mmap[i + 2] == 0xFF {
            if let Ok(img) = image::load_from_memory(&mmap[i..]) {
                let s = img.width() * img.height();
                if s > best_size {
                    best_size = s;
                    best_img = Some(img);
                }
                // Skip ahead 2KB to find next potential candidate
                i += 2048;
                continue;
            }
        }
        i += 1;
    }

    best_img.ok_or_else(|| "No decodable JPEG preview found via brute force".into())
}

/// Variant that returns raw bytes, needed for extraction without re-encoding to serve as Web preview.
pub fn brute_force_extract_jpeg_data(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = brute_force_scan_jpeg(path)?;
    let mut jpeg_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_data);

    let rgb = img.to_rgb8();
    rgb.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
    Ok(jpeg_data)
}

/// Extracts the largest embedded preview from a RAW file using rsraw.
pub fn extract_raw_preview_data(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Load the RAW file
    let file = std::fs::File::open(path)?;

    // Use memory mapping instead of reading entire file to heap
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };

    let mut raw =
        rsraw::RawImage::open(&mmap).map_err(|e| format!("LibRaw open error: {:?}", e))?;

    let thumbs = raw
        .extract_thumbs()
        .map_err(|e| format!("LibRaw extract_thumbs error: {:?}", e))?;

    // Find the largest thumbnail
    if let Some(thumb) = thumbs.iter().max_by_key(|t| t.width * t.height) {
        if thumb.data.is_empty() {
            return Err("Extracted thumbnail is empty".into());
        }
        Ok(thumb.data.clone())
    } else {
        Err("No embedded thumbnails found in RAW file".into())
    }
}

/// Helper to resize and save the image
pub(crate) fn process_image(
    img: image::DynamicImage,
    output_path: &Path,
    size_px: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::thumbnails::native::encode_webp_native;
    use fast_image_resize as fr;

    let width = img.width();
    let height = img.height();

    // Calculate target dimensions
    let aspect = width as f32 / height as f32;
    let (new_w, new_h) = if aspect > 1.0 {
        (size_px, (size_px as f32 / aspect).max(1.0) as u32)
    } else {
        (((size_px as f32 * aspect).max(1.0)) as u32, size_px)
    };

    // Prepare source for resizing
    let src_image = fr::images::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .map_err(|e| e.to_string())?;

    // Resize
    let mut dst_image = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);
    let mut resizer = fr::Resizer::new();
    let options =
        fr::ResizeOptions::new().resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear));

    resizer
        .resize(&src_image, &mut dst_image, Some(&options))
        .map_err(|e| e.to_string())?;

    // Save as WebP
    let buffer = dst_image.buffer();
    encode_webp_native(buffer, new_w, new_h, output_path)?;

    Ok(())
}
