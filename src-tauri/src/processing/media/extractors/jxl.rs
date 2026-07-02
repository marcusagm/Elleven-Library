//! Native JPEG XL (JXL) extraction logic using `jxl-oxide`.
//!
//! This module provides pure-Rust JXL decoding, completely eliminating the
//! dependency on FFmpeg having `libjxl` compiled in. The `jxl-oxide` crate
//! is a 100% safe Rust decoder that supports:
//!
//! - Both bare codestream (`FF 0A`) and ISOBMFF container formats
//! - VarDCT and Modular compression modes
//! - HDR images with float precision (tone-mapped to 8-bit sRGB on output)
//! - Animated JXL (multi-frame — first keyframe extracted)
//! - EXIF/XMP metadata in containerized files
//!
//! # Extraction Tiers
//!
//! 1. **jxl-oxide native** — Pure Rust decode → resize → WebP (primary)
//! 2. **FFmpeg fallback** — For edge cases where jxl-oxide may fail

use crate::core::error::{AppError, AppResult};
use crate::processing::media::extractors::image::process_and_encode_webp;
use std::path::Path;

/// Converts a linear float sample (potentially HDR, range 0..∞) to an sRGB u8 value.
///
/// Applies a simple Reinhard tone-mapping for HDR values followed by the sRGB
/// gamma curve (approximate) to produce perceptually correct 8-bit output.
#[inline]
fn float_to_srgb_u8(linear_value: f32) -> u8 {
    // Reinhard tone-mapping: maps [0..∞) → [0..1)
    let tone_mapped = if linear_value > 1.0 {
        linear_value / (1.0 + linear_value)
    } else {
        linear_value.max(0.0)
    };

    // Apply approximate sRGB gamma curve
    let srgb_value = if tone_mapped <= 0.0031308 {
        tone_mapped * 12.92
    } else {
        1.055 * tone_mapped.powf(1.0 / 2.4) - 0.055
    };

    (srgb_value.clamp(0.0, 1.0) * 255.0) as u8
}

/// Decodes a JXL file into an `image::DynamicImage` using `jxl-oxide`.
///
/// This function handles both bare codestream and ISOBMFF container formats.
/// For animated JXL files, only the first keyframe is decoded.
///
/// Uses `image_all_channels()` which returns a `FrameBuffer` with f32 data
/// (orientation-applied, interleaved). The float data is then tone-mapped
/// to sRGB u8 for downstream compatibility.
///
/// # Arguments
///
/// * `path` - Path to the JXL file.
///
/// # Errors
///
/// * `AppError::Generic` - If decoding or pixel conversion fails.
fn decode_jxl_to_dynamic_image(path: &Path) -> AppResult<image::DynamicImage> {
    use jxl_oxide::JxlImage;

    let jxl_image = JxlImage::builder().open(path).map_err(|decode_error| {
        AppError::Generic(format!(
            "jxl-oxide failed to open {:?}: {}",
            path, decode_error
        ))
    })?;

    let image_header = jxl_image.image_header();
    let image_width = image_header.size.width;
    let image_height = image_header.size.height;

    if image_width == 0 || image_height == 0 {
        return Err(AppError::Generic(format!(
            "JXL image has zero dimensions: {}x{} in {:?}",
            image_width, image_height, path
        )));
    }

    // Render the first keyframe (index 0).
    // render_frame returns Result<Render> directly.
    let rendered_frame = jxl_image.render_frame(0).map_err(|render_error| {
        AppError::Generic(format!(
            "jxl-oxide failed to render frame 0 of {:?}: {}",
            path, render_error
        ))
    })?;

    // Use image_all_channels() which returns FrameBuffer with:
    // - orientation applied
    // - all channels interleaved as f32
    // - buf() returns &[f32] with length = width * height * channels
    let frame_buffer = rendered_frame.image_all_channels();
    let rendered_width = frame_buffer.width() as u32;
    let rendered_height = frame_buffer.height() as u32;
    let channel_count = frame_buffer.channels();
    let float_pixels = frame_buffer.buf();

    // Convert f32 float data to RGBA u8 with tone-mapping
    let pixel_count = rendered_width as usize * rendered_height as usize;
    let mut rgba_bytes = Vec::with_capacity(pixel_count * 4);

    match channel_count {
        4 => {
            // RGBA
            for pixel_chunk in float_pixels.chunks_exact(4) {
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[0]));
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[1]));
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[2]));
                rgba_bytes.push((pixel_chunk[3].clamp(0.0, 1.0) * 255.0) as u8);
            }
        }
        3 => {
            // RGB
            for pixel_chunk in float_pixels.chunks_exact(3) {
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[0]));
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[1]));
                rgba_bytes.push(float_to_srgb_u8(pixel_chunk[2]));
                rgba_bytes.push(255);
            }
        }
        2 => {
            // Grayscale + Alpha
            for pixel_chunk in float_pixels.chunks_exact(2) {
                let gray_value = float_to_srgb_u8(pixel_chunk[0]);
                rgba_bytes.push(gray_value);
                rgba_bytes.push(gray_value);
                rgba_bytes.push(gray_value);
                rgba_bytes.push((pixel_chunk[1].clamp(0.0, 1.0) * 255.0) as u8);
            }
        }
        1 => {
            // Grayscale
            for sample in float_pixels {
                let gray_value = float_to_srgb_u8(*sample);
                rgba_bytes.push(gray_value);
                rgba_bytes.push(gray_value);
                rgba_bytes.push(gray_value);
                rgba_bytes.push(255);
            }
        }
        5 => {
            // CMYK + Alpha — naive CMYK→RGB conversion
            for pixel_chunk in float_pixels.chunks_exact(5) {
                let cyan = pixel_chunk[0].clamp(0.0, 1.0);
                let magenta = pixel_chunk[1].clamp(0.0, 1.0);
                let yellow = pixel_chunk[2].clamp(0.0, 1.0);
                let black_key = pixel_chunk[3].clamp(0.0, 1.0);
                let alpha = pixel_chunk[4].clamp(0.0, 1.0);

                rgba_bytes.push(((1.0 - cyan) * (1.0 - black_key) * 255.0) as u8);
                rgba_bytes.push(((1.0 - magenta) * (1.0 - black_key) * 255.0) as u8);
                rgba_bytes.push(((1.0 - yellow) * (1.0 - black_key) * 255.0) as u8);
                rgba_bytes.push((alpha * 255.0) as u8);
            }
        }
        _ => {
            // Unknown channel count — take the first 3 or less channels as RGB
            if channel_count >= 3 {
                for pixel_chunk in float_pixels.chunks_exact(channel_count) {
                    rgba_bytes.push(float_to_srgb_u8(pixel_chunk[0]));
                    rgba_bytes.push(float_to_srgb_u8(pixel_chunk[1]));
                    rgba_bytes.push(float_to_srgb_u8(pixel_chunk[2]));
                    rgba_bytes.push(255);
                }
            } else {
                return Err(AppError::Generic(format!(
                    "Unsupported JXL channel count: {} in {:?}",
                    channel_count, path
                )));
            }
        }
    }

    image::RgbaImage::from_raw(rendered_width, rendered_height, rgba_bytes)
        .map(image::DynamicImage::ImageRgba8)
        .ok_or_else(|| {
            AppError::Generic(format!(
                "Failed to construct RGBA image from JXL data ({}x{}, {} channels) in {:?}",
                rendered_width, rendered_height, channel_count, path
            ))
        })
}

// ─── Public API ────────────────────────────────────────────────────────────

/// Extracts technical metadata from a JXL file using `jxl-oxide` for dimensions
/// and `rexif` for EXIF data (containerized JXL files with EXIF boxes).
///
/// # Arguments
///
/// * `path` - Path to the JXL file.
///
/// # Errors
///
/// * `AppError::Generic` - If the JXL header cannot be parsed.
pub fn extract_jxl_metadata(path: &Path) -> AppResult<serde_json::Value> {
    use jxl_oxide::JxlImage;

    let jxl_image = JxlImage::builder().open(path).map_err(|decode_error| {
        AppError::Generic(format!(
            "jxl-oxide failed to open {:?} for metadata: {}",
            path, decode_error
        ))
    })?;

    let image_header = jxl_image.image_header();
    let image_width = image_header.size.width;
    let image_height = image_header.size.height;

    let is_animation = image_header.metadata.animation.is_some();
    let bit_depth = image_header.metadata.bit_depth.bits_per_sample();

    let color_encoding_description = format!("{:?}", image_header.metadata.colour_encoding);

    let mut metadata = serde_json::json!({
        "width": image_width,
        "height": image_height,
        "format": "JPEG XL",
        "codec": "jxl",
        "is_animation": is_animation,
        "bit_depth": bit_depth,
        "color_encoding": color_encoding_description,
    });

    // Try to extract EXIF data via rexif (works for ISOBMFF-containerized JXL files)
    if let Ok(exif_data) = rexif::parse_file(path) {
        if let Some(metadata_object) = metadata.as_object_mut() {
            for entry in exif_data.entries {
                let tag_name = entry.tag.to_string();
                let tag_value = entry.value_more_readable.to_string();
                if !tag_value.trim().is_empty() {
                    metadata_object.insert(tag_name, serde_json::Value::String(tag_value));
                }
            }
        }
    }

    Ok(metadata)
}

/// Generates a WebP thumbnail from a JXL file using native `jxl-oxide` decoding.
///
/// **Tier 1**: Native jxl-oxide decode → resize → WebP encode.
/// **Tier 2**: FFmpeg fallback for edge cases.
///
/// # Arguments
///
/// * `path` - Path to the JXL file.
/// * `size_hint` - Maximum dimension (width or height) in pixels.
///
/// # Errors
///
/// * `AppError::Generic` - If all tiers fail.
pub fn generate_jxl_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    // Tier 1: Native jxl-oxide decode
    if let Ok(decoded_image) = decode_jxl_to_dynamic_image(path) {
        if let Ok(webp_bytes) = process_and_encode_webp(decoded_image, size_hint) {
            return Ok(webp_bytes);
        }
    }

    // Tier 2: FFmpeg fallback
    if let Ok(ffmpeg_bytes) =
        crate::processing::media::extractors::image::generate_ffmpeg_image_thumbnail(
            path, size_hint,
        )
    {
        return Ok(ffmpeg_bytes);
    }

    Err(AppError::Generic(format!(
        "All JXL thumbnail generation tiers failed for {:?}",
        path
    )))
}

/// Generates a high-quality preview from a JXL file using native `jxl-oxide` decoding.
///
/// **Tier 1**: Native jxl-oxide decode → resize to max 2048px → WebP encode.
/// **Tier 2**: FFmpeg fallback.
///
/// # Arguments
///
/// * `path` - Path to the JXL file.
///
/// # Returns
///
/// `AppResult<(Vec<u8>, String)>` - Preview bytes and MIME type.
///
/// # Errors
///
/// * `AppError::Generic` - If all tiers fail.
pub fn extract_jxl_preview(path: &Path) -> AppResult<(Vec<u8>, String)> {
    // Tier 1: Native jxl-oxide decode → WebP preview
    if let Ok(decoded_image) = decode_jxl_to_dynamic_image(path) {
        if let Ok(webp_bytes) = process_and_encode_webp(decoded_image, 2048) {
            return Ok((webp_bytes, "image/webp".to_string()));
        }
    }

    // Tier 2: FFmpeg fallback
    if let Ok(ffmpeg_result) =
        crate::processing::media::extractors::image::generate_ffmpeg_image_preview(path)
    {
        return Ok(ffmpeg_result);
    }

    Err(AppError::Generic(format!(
        "All JXL preview generation tiers failed for {:?}",
        path
    )))
}
