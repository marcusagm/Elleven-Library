//! Shared image extraction logic for all raster, RAW and modern image providers.
//!
//! This module centralises thumbnail generation and metadata extraction so every
//! per-format provider in `providers/image/` can delegate to a single, well-tested
//! implementation rather than duplicating logic.
//!
//! # Extraction Tiers (RAW)
//!
//! 1. **LibRaw** (`rsraw`) — extracts the largest embedded preview.
//! 2. **Brute-force JPEG scan** — walks the file looking for `FF D8 FF` markers.
//! 3. **FFmpeg fallback** — last resort for formats that embed no preview.

use crate::core::error::{AppError, AppResult};
use std::path::Path;

// ─── Shared helpers ────────────────────────────────────────────────────────

/// Resizes a [`image::DynamicImage`] to fit within `size_hint` and encodes the
/// result as WebP at 80 % quality.
///
/// # Arguments
///
/// * `source_image` - The decoded source image.
/// * `size_hint` - Maximum dimension (width or height) in pixels.
///
/// # Errors
///
/// * `AppError::Generic` - If resize or WebP encoding fails.
pub fn process_and_encode_webp(
    source_image: image::DynamicImage,
    size_hint: u32,
) -> AppResult<Vec<u8>> {
    use fast_image_resize as fr;

    let source_width = source_image.width();
    let source_height = source_image.height();

    if source_width == 0 || source_height == 0 {
        return Err(AppError::Generic(
            "Image dimensions are zero; cannot generate thumbnail".into(),
        ));
    }

    let aspect_ratio = source_width as f32 / source_height as f32;
    let (target_width, target_height) = if aspect_ratio > 1.0 {
        (
            size_hint,
            (size_hint as f32 / aspect_ratio).max(1.0) as u32,
        )
    } else {
        (
            (size_hint as f32 * aspect_ratio).max(1.0) as u32,
            size_hint,
        )
    };

    let source_fr_image = fr::images::Image::from_vec_u8(
        source_width,
        source_height,
        source_image.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .map_err(|error| AppError::Generic(error.to_string()))?;

    let mut destination_fr_image =
        fr::images::Image::new(target_width, target_height, fr::PixelType::U8x4);

    let mut resizer = fr::Resizer::new();
    let resize_options = fr::ResizeOptions::new()
        .resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Bilinear));

    resizer
        .resize(&source_fr_image, &mut destination_fr_image, Some(&resize_options))
        .map_err(|error| AppError::Generic(error.to_string()))?;

    let webp_encoder =
        webp::Encoder::from_rgba(destination_fr_image.buffer(), target_width, target_height);
    let webp_data = webp_encoder.encode(80.0);

    Ok(webp_data.to_vec())
}

// ─── EXIF helpers ──────────────────────────────────────────────────────────

/// Extracts EXIF metadata from any image file that embeds it (JPEG, TIFF, RAW, etc.).
///
/// Returns a `serde_json::Map` with human-readable keys and string values. An
/// empty map is returned if no EXIF data is found or parsing fails.
///
/// # Arguments
///
/// * `path` - Path to the image file.
fn extract_exif_data(path: &Path) -> serde_json::Map<String, serde_json::Value> {
    let mut exif_map = serde_json::Map::new();

    if let Ok(exif_data) = rexif::parse_file(path) {
        for entry in exif_data.entries {
            let tag_name = entry.tag.to_string();
            let tag_value = entry.value_more_readable.to_string();
            if !tag_value.trim().is_empty() {
                exif_map.insert(tag_name, serde_json::Value::String(tag_value));
            }
        }
    }

    exif_map
}

/// Parses a resolution value string from EXIF into an `f64`.
///
/// EXIF resolution values are often expressed as rational numbers like `"72/1"`.
fn parse_exif_resolution_value(resolution_string: &str) -> Option<f64> {
    let trimmed = resolution_string.trim();
    if let Some(slash_position) = trimmed.find('/') {
        let numerator: f64 = trimmed[..slash_position].trim().parse().ok()?;
        let denominator: f64 = trimmed[slash_position + 1..].trim().parse().ok()?;
        if denominator != 0.0 {
            return Some(numerator / denominator);
        }
    }
    trimmed.parse::<f64>().ok()
}

// ─── Standard raster ───────────────────────────────────────────────────────

/// Extracts technical metadata from a standard raster image (JPEG, PNG, GIF,
/// BMP, TIFF, DDS, NetPBM, etc.) using the `image` crate for dimensions and
/// `rexif` for EXIF data including resolution (DPI).
///
/// # Arguments
///
/// * `path` - Path to the image file.
///
/// # Errors
///
/// * `AppError::Io` - If the file cannot be opened.
/// * `AppError::Generic` - If dimension extraction fails.
pub fn extract_raster_metadata(path: &Path) -> AppResult<serde_json::Value> {
    let image_reader = image::ImageReader::open(path)
        .map_err(AppError::Io)?
        .with_guessed_format()
        .map_err(AppError::Io)?;

    let detected_format = image_reader.format();
    let image_dimensions = match image_reader.into_dimensions() {
        Ok(dimensions) => dimensions,
        Err(dimension_error) => {
            tracing::warn!("ImageReader::into_dimensions failed ({}), falling back to full decode: {:?}", dimension_error, path);
            let decoded_image = image::open(path)
                .map_err(|decode_error| AppError::Generic(format!("Image dimension/decode fallback error: {}", decode_error)))?;
            (decoded_image.width(), decoded_image.height())
        }
    };

    let exif_map = extract_exif_data(path);

    let x_resolution = exif_map
        .get("XResolution")
        .and_then(|value| value.as_str())
        .and_then(parse_exif_resolution_value);

    let y_resolution = exif_map
        .get("YResolution")
        .and_then(|value| value.as_str())
        .and_then(parse_exif_resolution_value);

    let resolution_unit = exif_map
        .get("ResolutionUnit")
        .and_then(|value| value.as_str())
        .map(|unit| unit.to_string());

    let mut metadata = serde_json::json!({
        "width": image_dimensions.0,
        "height": image_dimensions.1,
        "format": detected_format.map(|format| format!("{:?}", format)).unwrap_or_default(),
    });

    if let Some(obj) = metadata.as_object_mut() {
        for (k, v) in exif_map {
            obj.insert(k, v);
        }
    }

    if let Some(dpi_horizontal) = x_resolution {
        metadata["x_resolution"] = serde_json::json!(dpi_horizontal);
    }
    if let Some(dpi_vertical) = y_resolution {
        metadata["y_resolution"] = serde_json::json!(dpi_vertical);
    }
    if let Some(unit) = resolution_unit {
        metadata["resolution_unit"] = serde_json::json!(unit);
    }

    Ok(metadata)
}

/// Generates a WebP thumbnail from a standard raster image.
///
/// Uses `zune-jpeg` for fast JPEG decoding and `image-rs` for all other formats,
/// then resizes with `fast-image-resize` and encodes to WebP.
///
/// # Arguments
///
/// * `path` - Path to the image file.
/// * `size_hint` - Maximum dimension (width or height) in pixels.
///
/// # Errors
///
/// * `AppError::Io` - If the file cannot be read.
/// * `AppError::Generic` - If decoding, resizing or encoding fails.
pub fn generate_raster_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    use zune_jpeg::JpegDecoder;

    let file_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_lowercase();

    let decoded_image = match file_extension.as_str() {
        "jpg" | "jpeg" | "jpe" | "jfif" => {
            let jpeg_bytes =
                std::fs::read(path).map_err(AppError::Io)?;
            let mut jpeg_decoder = JpegDecoder::new(&jpeg_bytes);
            let decoded_pixels = jpeg_decoder
                .decode()
                .map_err(|error| AppError::Generic(format!("JPEG decode error: {:?}", error)))?;
            let jpeg_info = jpeg_decoder.info().ok_or_else(|| {
                AppError::Generic("Failed to retrieve JPEG image info".into())
            })?;

            let mut rgba_pixels = Vec::with_capacity(decoded_pixels.len() / 3 * 4);
            for rgb_chunk in decoded_pixels.chunks_exact(3) {
                rgba_pixels.push(rgb_chunk[0]);
                rgba_pixels.push(rgb_chunk[1]);
                rgba_pixels.push(rgb_chunk[2]);
                rgba_pixels.push(255);
            }

            let rgba_buffer = image::RgbaImage::from_raw(
                jpeg_info.width as u32,
                jpeg_info.height as u32,
                rgba_pixels,
            )
            .ok_or_else(|| AppError::Generic("Failed to build RGBA buffer from JPEG data".into()))?;
            image::DynamicImage::ImageRgba8(rgba_buffer)
        }
        _ => image::open(path)
            .map_err(|error| AppError::Generic(format!("Image decode error: {}", error)))?,
    };

    process_and_encode_webp(decoded_image, size_hint)
}

// ─── RAW photography ───────────────────────────────────────────────────────

/// Extracts technical metadata from a RAW photography file.
///
/// Uses LibRaw (`rsraw`) for canonical image dimensions and then `rexif` to
/// surface camera-specific EXIF data (make, model, exposure settings, etc.).
///
/// # Arguments
///
/// * `path` - Path to the RAW file.
///
/// # Errors
///
/// * `AppError::Io` - If the file cannot be opened.
/// * `AppError::Generic` - If LibRaw parsing fails.
pub fn extract_raw_metadata(path: &Path) -> AppResult<serde_json::Value> {
    let mut image_width = 0u32;
    let mut image_height = 0u32;
    
    // First try LibRaw
    if let Ok(file_handle) = std::fs::File::open(path) {
        if let Ok(memory_map) = unsafe { memmap2::MmapOptions::new().map(&file_handle) } {
            if let Ok(raw_image) = rsraw::RawImage::open(&memory_map) {
                image_width = raw_image.width();
                image_height = raw_image.height();
            }
        }
    }
    
    // If LibRaw fails, fallback to brute force dimension extraction
    if image_width == 0 || image_height == 0 {
        if let Ok((bytes, _)) = brute_force_extract_jpeg_bytes(path) {
            if let Ok(decoded_image) = image::load_from_memory(&bytes) {
                image_width = decoded_image.width();
                image_height = decoded_image.height();
            }
        }
    }

    // Supplement LibRaw dimensions with EXIF camera metadata via rexif.
    // Most RAW formats (ARW, CR2, NEF, DNG, etc.) are TIFF-based and expose EXIF.
    let exif_map = extract_exif_data(path);

    let x_resolution = exif_map
        .get("XResolution")
        .and_then(|value| value.as_str())
        .and_then(parse_exif_resolution_value);

    let y_resolution = exif_map
        .get("YResolution")
        .and_then(|value| value.as_str())
        .and_then(parse_exif_resolution_value);

    let mut metadata = serde_json::json!({
        "width": image_width,
        "height": image_height,
    });

    if let Some(obj) = metadata.as_object_mut() {
        for (k, v) in exif_map {
            obj.insert(k, v);
        }
    }

    if let Some(dpi_horizontal) = x_resolution {
        metadata["x_resolution"] = serde_json::json!(dpi_horizontal);
    }
    if let Some(dpi_vertical) = y_resolution {
        metadata["y_resolution"] = serde_json::json!(dpi_vertical);
    }

    Ok(metadata)
}

fn ensure_jpeg_bytes(raw_bytes: Vec<u8>) -> AppResult<Vec<u8>> {
    if raw_bytes.starts_with(&[0xFF, 0xD8]) {
        // Already a JPEG
        Ok(raw_bytes)
    } else if raw_bytes.starts_with(&[0x49, 0x49, 0x2A, 0x00])
        || raw_bytes.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        || raw_bytes.starts_with(&[0x89, b'P', b'N', b'G'])
    {
        // TIFF or PNG, convert to JPEG
        let decoded_image = image::load_from_memory(&raw_bytes)
            .map_err(|decode_error| AppError::Generic(format!("Failed to decode embedded raw preview image: {}", decode_error)))?;
        let mut jpeg_data = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut jpeg_data);
        decoded_image
            .to_rgb8()
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|encode_error| AppError::Generic(format!("Failed to encode raw preview to JPEG: {}", encode_error)))?;
        Ok(jpeg_data)
    } else {
        // Fallback: try decoding anyway, if it works convert to JPEG
        if let Ok(decoded_image) = image::load_from_memory(&raw_bytes) {
            let mut jpeg_data = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut jpeg_data);
            if decoded_image.to_rgb8().write_to(&mut cursor, image::ImageFormat::Jpeg).is_ok() {
                return Ok(jpeg_data);
            }
        }
        Ok(raw_bytes)
    }
}

/// Generates a WebP thumbnail from a RAW photography file using a hybrid strategy.
///
/// **Tier 0**: quickraw high-fidelity extraction.
/// **Tier 1**: LibRaw embedded preview extraction (fastest, best quality).
/// **Tier 2**: Brute-force JPEG scan (scans the first 30 MB for `FF D8 FF` markers).
/// **Tier 3**: FFmpeg conversion fallback (for formats without embedded JPEG previews).
///
/// # Arguments
///
/// * `path` - Path to the RAW file.
/// * `size_hint` - Maximum dimension (width or height) in pixels.
///
/// # Errors
///
/// * `AppError::Generic` - If all extraction tiers fail.
pub fn generate_raw_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    // Tier 0: quickraw high-fidelity extraction
    let path_clone = path.to_path_buf();
    let quickraw_result = std::panic::catch_unwind(|| {
        if let Ok(raw_data) = std::fs::read(&path_clone) {
            return quickraw::Export::export_thumbnail_data(&raw_data)
                .map(|(data, _)| data.to_vec())
                .ok();
        }
        None
    });

    if let Ok(Some(thumbnail_data)) = quickraw_result {
        if let Ok(decoded_preview) = image::load_from_memory(&thumbnail_data) {
            return process_and_encode_webp(decoded_preview, size_hint);
        }
    }

    // Tier 1: LibRaw embedded preview
    if let Ok(embedded_preview_bytes) = extract_libraw_preview(path) {
        if let Ok(decoded_preview) = image::load_from_memory(&embedded_preview_bytes) {
            return process_and_encode_webp(decoded_preview, size_hint);
        }
    }

    // Tier 2: Brute-force JPEG scan
    if let Ok((embedded_preview_bytes, _)) = brute_force_extract_jpeg_bytes(path) {
        if let Ok(decoded_preview) = image::load_from_memory(&embedded_preview_bytes) {
            return process_and_encode_webp(decoded_preview, size_hint);
        }
    }

    // Tier 3: FFmpeg fallback (handles formats like x3f, gpr, iiq that lack embedded JPEGs)
    if let Ok(ffmpeg_bytes) = generate_ffmpeg_image_thumbnail(path, size_hint) {
        return Ok(ffmpeg_bytes);
    }

    Err(AppError::Generic(format!(
        "All RAW extraction tiers failed for {:?}",
        path
    )))
}

/// Extracts a high-quality embedded JPEG preview from a RAW file for previewing.
/// It returns the raw bytes directly without WebP re-encoding to preserve quality and speed.
pub fn extract_raw_preview(path: &Path) -> AppResult<Vec<u8>> {
    // Tier 0: quickraw high-fidelity extraction
    let path_clone = path.to_path_buf();
    let quickraw_result = std::panic::catch_unwind(|| {
        if let Ok(raw_data) = std::fs::read(&path_clone) {
            return quickraw::Export::export_thumbnail_data(&raw_data)
                .map(|(data, _)| data.to_vec())
                .ok();
        }
        None
    });

    if let Ok(Some(thumbnail_data)) = quickraw_result {
        if let Ok(jpeg_bytes) = ensure_jpeg_bytes(thumbnail_data) {
            return Ok(jpeg_bytes);
        }
    }

    // Tier 1: LibRaw embedded preview
    if let Ok(embedded_preview_bytes) = extract_libraw_preview(path) {
        if let Ok(jpeg_bytes) = ensure_jpeg_bytes(embedded_preview_bytes) {
            return Ok(jpeg_bytes);
        }
    }

    // Tier 2: Brute-force JPEG scan
    if let Ok((embedded_preview_bytes, _)) = brute_force_extract_jpeg_bytes(path) {
        if let Ok(jpeg_bytes) = ensure_jpeg_bytes(embedded_preview_bytes) {
            return Ok(jpeg_bytes);
        }
    }

    // Tier 3: FFmpeg fallback (extracts full size or max 2048px JPEG)
    if let Ok((ffmpeg_bytes, _)) = generate_ffmpeg_image_preview(path) {
        return Ok(ffmpeg_bytes);
    }

    Err(AppError::Generic(format!(
        "Could not extract preview from RAW file {:?}",
        path
    )))
}

/// Generates a WebP thumbnail for HDR/EXR/DDS files using a multi-tier fallback strategy.
pub fn generate_hdr_exr_dds_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    // Tier 1: Try raster image decoding first (native image-rs)
    if let Ok(thumbnail_bytes) = generate_raster_thumbnail(path, size_hint) {
        return Ok(thumbnail_bytes);
    }
    // Tier 2: Try FFmpeg extraction fallback
    if let Ok(thumbnail_bytes) = generate_ffmpeg_image_thumbnail(path, size_hint) {
        return Ok(thumbnail_bytes);
    }
    // Tier 3: Try brute force JPEG scanner
    if let Ok((image_bytes, _)) = brute_force_extract_jpeg_bytes(path) {
        if let Ok(decoded_image) = image::load_from_memory(&image_bytes) {
            return process_and_encode_webp(decoded_image, size_hint);
        }
    }
    Err(AppError::Generic(format!("Failed to generate thumbnail for HDR/EXR/DDS file: {:?}", path)))
}

/// Generates a preview for HDR/EXR/DDS files using a multi-tier fallback strategy.
pub fn generate_hdr_exr_dds_preview(path: &Path) -> AppResult<(Vec<u8>, String)> {
    // Tier 1: Try raster preview (native image-rs)
    if let Ok(preview_result) = generate_raster_preview(path) {
        return Ok(preview_result);
    }
    // Tier 2: Try FFmpeg preview fallback
    if let Ok(preview_result) = generate_ffmpeg_image_preview(path) {
        return Ok(preview_result);
    }
    // Tier 3: Try brute force JPEG scanner
    if let Ok((jpeg_bytes, _)) = brute_force_extract_jpeg_bytes(path) {
        return Ok((jpeg_bytes, "image/jpeg".to_string()));
    }
    Err(AppError::Generic(format!("Failed to generate preview for HDR/EXR/DDS file: {:?}", path)))
}


/// Extracts the largest embedded JPEG preview from a RAW file using LibRaw.
fn extract_libraw_preview(path: &Path) -> AppResult<Vec<u8>> {
    let file_handle = std::fs::File::open(path).map_err(AppError::Io)?;
    let memory_map = unsafe {
        memmap2::MmapOptions::new()
            .map(&file_handle)
            .map_err(AppError::Io)?
    };

    let mut raw_image = rsraw::RawImage::open(&memory_map).map_err(|error| {
        AppError::Generic(format!("LibRaw open error: {:?}", error))
    })?;

    let embedded_thumbnails = raw_image.extract_thumbs().map_err(|error| {
        AppError::Generic(format!("LibRaw thumb extraction error: {:?}", error))
    })?;

    embedded_thumbnails
        .iter()
        .max_by_key(|thumbnail| thumbnail.width * thumbnail.height)
        .map(|thumbnail| thumbnail.data.clone())
        .ok_or_else(|| AppError::Generic("No embedded thumbnails found in RAW file".into()))
}

/// Scans the first 8 MB of a file looking for JPEG start-of-image markers and
/// returns the largest valid JPEG image found.
fn brute_force_extract_jpeg_bytes(path: &Path) -> AppResult<(Vec<u8>, u32)> {
    let file_handle = std::fs::File::open(path).map_err(AppError::Io)?;
    let memory_map = unsafe {
        memmap2::MmapOptions::new()
            .map(&file_handle)
            .map_err(AppError::Io)?
    };

    // Scan up to 30MB like in V1
    let scan_limit = memory_map.len().min(30 * 1024 * 1024);
    let mut best_bytes: Option<Vec<u8>> = None;
    let mut best_pixel_count = 0u32;
    let mut scan_offset = 0usize;

    while scan_offset < scan_limit.saturating_sub(4) {
        let is_jpeg_marker = memory_map[scan_offset] == 0xFF
            && memory_map[scan_offset + 1] == 0xD8
            && memory_map[scan_offset + 2] == 0xFF;

        if is_jpeg_marker {
            if let Ok(decoded_image) = image::load_from_memory(&memory_map[scan_offset..]) {
                let pixel_count = decoded_image.width() * decoded_image.height();
                if pixel_count > best_pixel_count {
                    best_pixel_count = pixel_count;
                    // Copy the slice correctly by finding EOI, or just storing the slice
                    // Since image::load_from_memory stops at the end of the image, we don't know the exact byte length.
                    // But we can scan forward for EOI (FF D9).
                    let eoi_limit = (scan_offset + 20 * 1024 * 1024).min(memory_map.len());
                    let mut end_offset = scan_offset + 2;
                    let mut found_eoi = false;
                    while end_offset < eoi_limit.saturating_sub(1) {
                        if memory_map[end_offset] == 0xFF && memory_map[end_offset + 1] == 0xD9 {
                            end_offset += 2;
                            found_eoi = true;
                            break;
                        }
                        end_offset += 1;
                    }
                    
                    if found_eoi {
                        best_bytes = Some(memory_map[scan_offset..end_offset].to_vec());
                    } else {
                        // Fallback: just save it as JPEG again
                        let mut jpeg_data = Vec::new();
                        let mut cursor = std::io::Cursor::new(&mut jpeg_data);
                        if decoded_image.to_rgb8().write_to(&mut cursor, image::ImageFormat::Jpeg).is_ok() {
                            best_bytes = Some(jpeg_data);
                        }
                    }
                }
                scan_offset += 2048;
                continue;
            }
        }
        scan_offset += 1;
    }

    if let Some(bytes) = best_bytes {
        Ok((bytes, best_pixel_count))
    } else {
        Err(AppError::Generic("Brute-force JPEG scan found no valid images".into()))
    }
}

// ─── FFmpeg-based (modern formats) ─────────────────────────────────────────

/// Extracts technical metadata from a modern image format (HEIC, AVIF, JXL)
/// using `ffprobe`.
///
/// # Arguments
///
/// * `path` - Path to the image file.
///
/// # Errors
///
/// * `AppError::Generic` - If `ffprobe` is not found or returns an error.
/// * `AppError::Transcoding` - If JSON parsing of `ffprobe` output fails.
pub fn extract_ffmpeg_image_metadata(path: &Path) -> AppResult<serde_json::Value> {
    use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
    use std::process::Command;

    let transcoding_tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

    let mut ffprobe_command = Command::new(transcoding_tools.ffprobe);
    ffprobe_command.args([
        "-v",
        "error",
        "-show_format",
        "-show_streams",
        "-of",
        "json",
        &path.to_string_lossy(),
    ]);

    let ffprobe_output = run_command_with_timeout(ffprobe_command, 10)?;

    if !ffprobe_output.status.success() {
        let error_message = String::from_utf8_lossy(&ffprobe_output.stderr);
        return Err(AppError::Transcoding(format!(
            "FFprobe failed: {}",
            error_message
        )));
    }

    let probe_json: serde_json::Value = serde_json::from_slice(&ffprobe_output.stdout)
        .map_err(|error| {
            AppError::Transcoding(format!("Failed to parse FFprobe JSON output: {}", error))
        })?;

    let mut technical_metadata = serde_json::Map::new();

    if let Some(streams) = probe_json.get("streams").and_then(|streams| streams.as_array()) {
        if let Some(first_stream) = streams.first() {
            if let Some(stream_width) = first_stream.get("width") {
                technical_metadata.insert("width".into(), stream_width.clone());
            }
            if let Some(stream_height) = first_stream.get("height") {
                technical_metadata.insert("height".into(), stream_height.clone());
            }
            if let Some(codec_name) = first_stream.get("codec_name") {
                technical_metadata.insert("codec".into(), codec_name.clone());
            }
            if let Some(color_space) = first_stream.get("color_space") {
                technical_metadata.insert("color_space".into(), color_space.clone());
            }
            if let Some(color_transfer) = first_stream.get("color_transfer") {
                technical_metadata.insert("color_transfer".into(), color_transfer.clone());
            }
        }
    }

    Ok(serde_json::Value::Object(technical_metadata))
}

/// Generates a WebP thumbnail from a modern image format using FFmpeg.
///
/// # Arguments
///
/// * `path` - Path to the image file.
/// * `size_hint` - Maximum dimension (width or height) in pixels.
///
/// # Errors
///
/// * `AppError::Transcoding` - If FFmpeg fails or times out.
pub fn generate_ffmpeg_image_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
    use std::process::Command;

    let transcoding_tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

    let mut ffmpeg_command = Command::new(transcoding_tools.ffmpeg);
    ffmpeg_command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        &path.to_string_lossy(),
        "-vf",
        &format!("scale='min({},iw)':'if(gt(ih,iw),-1,-2)':flags=lanczos", size_hint),
        "-vframes",
        "1",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
        "-",
    ]);

    // 60 seconds for heavy images like HEIC/AVIF
    let ffmpeg_output = run_command_with_timeout(ffmpeg_command, 60)?;

    if ffmpeg_output.status.success() {
        Ok(ffmpeg_output.stdout)
    } else {
        let error_message = String::from_utf8_lossy(&ffmpeg_output.stderr);
        Err(AppError::Transcoding(format!(
            "FFmpeg image thumbnail extraction failed: {}",
            error_message
        )))
    }
}

/// Generates a WebP preview from a generic raster image.
///
/// # Arguments
///
/// * `path` - Path to the image file.
///
/// # Errors
///
/// * `AppError::Generic` - If decoding or encoding fails.
pub fn generate_raster_preview(path: &Path) -> AppResult<(Vec<u8>, String)> {
    let decoded_image = image::open(path).map_err(|e| {
        AppError::Generic(format!("Raster preview extraction failed: {}", e))
    })?;

    let bytes = process_and_encode_webp(decoded_image, 2048)?;
    Ok((bytes, "image/webp".to_string()))
}

pub fn generate_ffmpeg_image_preview(path: &Path) -> AppResult<(Vec<u8>, String)> {
    use crate::processing::transcoding::{resolve_transcoding_tools, run_command_with_timeout};
    use std::process::Command;

    let transcoding_tools = resolve_transcoding_tools::<tauri::Wry>(None)?;

    let mut ffmpeg_command = Command::new(transcoding_tools.ffmpeg);
    ffmpeg_command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        &path.to_string_lossy(),
        "-vf",
        "scale='min(2048,iw)':'if(gt(ih,iw),-1,-2)':flags=lanczos",
        "-vframes",
        "1",
        "-f",
        "image2",
        "-c:v",
        "mjpeg",
        "-q:v",
        "2",
        "-",
    ]);

    // 60 seconds for heavy images like HEIC/AVIF
    let ffmpeg_output = run_command_with_timeout(ffmpeg_command, 60)?;

    if ffmpeg_output.status.success() {
        Ok((ffmpeg_output.stdout, "image/jpeg".to_string()))
    } else {
        let error_message = String::from_utf8_lossy(&ffmpeg_output.stderr);
        Err(AppError::Transcoding(format!(
            "FFmpeg image preview extraction failed: {}",
            error_message
        )))
    }
}

// ─── EXR ───────────────────────────────────────────────────────────────────

/// Extracts technical metadata from an OpenEXR file using the `image` crate.
///
/// # Arguments
///
/// * `path` - Path to the `.exr` file.
///
/// # Errors
///
/// * `AppError::Generic` - If the EXR file cannot be decoded.
pub fn extract_exr_metadata(path: &Path) -> AppResult<serde_json::Value> {
    let decoded_image = image::open(path)
        .map_err(|error| AppError::Generic(format!("EXR decode error: {}", error)))?;

    Ok(serde_json::json!({
        "width": decoded_image.width(),
        "height": decoded_image.height(),
        "color_mode": format!("{:?}", decoded_image.color()),
        "format": "OpenEXR",
    }))
}
