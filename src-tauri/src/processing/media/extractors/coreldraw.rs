//! CorelDRAW (.cdr) preview and metadata extractor.
//!
//! Supports three container formats found across CDR versions:
//! - **Modern ZIP** (CDR v16+): ZIP archive containing `previews/page1.png` or similar
//! - **Legacy RIFF** (CDR v7–v15): RIFF container with DISP/bmDt chunks
//! - **WL signature** (CDR v3–v5): Ancient binary format with 1-bit monochrome thumbnail
//!
//! Ported from V1 with full fidelity. All extraction paths produce `(bytes, mime_type)`.
//!
//! **V2 Pipeline Compatibility:** The thumbnail worker only accepts PNG, JPEG, and WebP.
//! BMP and TIFF outputs are automatically transcoded to PNG before returning.

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use serde_json::json;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Extracts the highest-quality preview image from a CorelDRAW file.
///
/// Dispatches to the appropriate strategy based on magic bytes, then runs
/// a fallback embedded-image scan for legacy formats.
///
/// # Errors
/// Returns error when no valid preview (>= 100 bytes) can be found.
pub fn extract_coreldraw_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    tracing::debug!("CDR: Analyzing file: {:?}", path);
    let mut candidates: Vec<(Vec<u8>, String)> = Vec::new();

    let mut file = File::open(path)?;
    let mut magic_bytes = [0u8; 4];
    if file.read_exact(&mut magic_bytes).is_ok() {
        if magic_bytes == [0x50, 0x4B, 0x03, 0x04] {
            tracing::debug!("CDR: Format detected: Modern ZIP");
            match extract_zip_best_quality(path) {
                Ok(result) => candidates.push(result),
                Err(error) => tracing::debug!("CDR: ZIP extraction failed: {}", error),
            }
        } else if magic_bytes == *b"RIFF" {
            tracing::debug!("CDR: Format detected: Legacy RIFF");
            match extract_riff_previews(path) {
                Ok(mut results) => candidates.append(&mut results),
                Err(error) => tracing::debug!("CDR: RIFF extraction failed: {}", error),
            }
        } else if magic_bytes[0] == 0x57 && magic_bytes[1] == 0x4C {
            tracing::debug!("CDR: Format detected: Legacy Corel (v3-v5) WL signature");
            if let Ok(thumbnail) = extract_wl_thumbnail(path) {
                candidates.push(thumbnail);
            }
        }
    }

    if candidates.is_empty() || is_legacy_format(path) {
        tracing::debug!("CDR: Running fallback BMP/Image scan...");
        if let Ok(mut results) = scan_for_embedded_images(path) {
            candidates.append(&mut results);
        }
    }

    if let Some(best) = candidates.into_iter().max_by_key(|(data, _)| data.len()) {
        tracing::debug!(
            "CDR: Best preview found. Size: {} bytes, mime: {}",
            best.0.len(),
            best.1
        );
        if best.0.len() < 100 {
            return Err("Preview too small to be valid".into());
        }
        // V2 pipeline only accepts PNG/JPEG/WebP — transcode BMP/TIFF to PNG
        return normalize_to_pipeline_format(best);
    }

    Err("No valid preview found in CorelDRAW file".into())
}

/// Extracts a high-resolution preview for the preview modal.
///
/// CDR embedded previews are typically small (88-256px). This function
/// extracts the best available source and upscales it to `target_size`
/// using Lanczos3 interpolation for smooth results in the UI.
///
/// # Errors
/// Returns error if no preview can be extracted or upscaling fails.
pub fn extract_coreldraw_preview_highres(
    path: &Path,
    target_size: u32,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let (source_data, _source_mime) = extract_coreldraw_preview(path)?;

    let source_image = image::load_from_memory(&source_data)
        .map_err(|error| format!("CDR: Failed to decode source for upscale: {}", error))?;

    let source_width = source_image.width();
    let source_height = source_image.height();

    // Skip upscale if source is already large enough
    if source_width >= target_size || source_height >= target_size {
        tracing::debug!(
            "CDR: Source {}x{} already meets target {}px, skipping upscale",
            source_width,
            source_height,
            target_size
        );
        return Ok((source_data, "image/png".to_string()));
    }

    let aspect_ratio = source_width as f64 / source_height as f64;
    let (target_width, target_height) = if aspect_ratio > 1.0 {
        (
            target_size,
            (target_size as f64 / aspect_ratio).max(1.0) as u32,
        )
    } else {
        (
            (target_size as f64 * aspect_ratio).max(1.0) as u32,
            target_size,
        )
    };

    tracing::debug!(
        "CDR: Upscaling preview {}x{} -> {}x{} with Lanczos3",
        source_width,
        source_height,
        target_width,
        target_height
    );

    let upscaled_image = source_image.resize(
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );

    let mut png_buffer = Vec::new();
    let mut cursor = Cursor::new(&mut png_buffer);
    upscaled_image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|error| format!("CDR: Failed to encode upscaled PNG: {}", error))?;

    tracing::debug!("CDR: Upscaled preview: {} bytes", png_buffer.len());
    Ok((png_buffer, "image/png".to_string()))
}

/// Normalizes image data to a format the V2 thumbnail pipeline accepts.
///
/// The thumbnail worker (`image_utils::detect_image_format`) only recognizes
/// PNG, JPEG, and WebP. BMP and TIFF data must be transcoded to PNG.
///
/// # Errors
/// Returns error if the image data cannot be decoded or re-encoded.
fn normalize_to_pipeline_format(
    (image_data, mime_type): (Vec<u8>, String),
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    match mime_type.as_str() {
        "image/png" | "image/jpeg" | "image/webp" => {
            // Already a pipeline-compatible format
            Ok((image_data, mime_type))
        }
        "image/bmp" | "image/tiff" | "image/gif" | _
            if mime_type != "image/png"
                && mime_type != "image/jpeg"
                && mime_type != "image/webp" =>
        {
            tracing::debug!(
                "CDR: Transcoding {} to PNG for pipeline compatibility",
                mime_type
            );
            let decoded_image = image::load_from_memory(&image_data)
                .map_err(|error| format!("CDR: Failed to decode {} image: {}", mime_type, error))?;
            let mut png_buffer = Vec::new();
            let mut cursor = Cursor::new(&mut png_buffer);
            decoded_image
                .write_to(&mut cursor, image::ImageFormat::Png)
                .map_err(|error| format!("CDR: Failed to encode PNG: {}", error))?;
            tracing::debug!(
                "CDR: Transcoded {} -> PNG ({} bytes)",
                mime_type,
                png_buffer.len()
            );
            Ok((png_buffer, "image/png".to_string()))
        }
        _ => Ok((image_data, mime_type)),
    }
}

/// Extracts **real document dimensions** from a CorelDRAW file.
///
/// For RIFF-based CDR files, reads the `mcfg` chunk which contains the actual
/// page size in CDR internal units (converted to millimeters).
/// For ZIP-based CDR files, extracts the internal `content/riffData.cdr` RIFF
/// and parses its `mcfg` chunk. Falls back to preview image dimensions.
///
/// # Returns
/// Tuple of (width, height) in millimeters as u32.
///
/// # Errors
/// Returns error if no dimension information can be extracted.
pub fn extract_coreldraw_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic_bytes = [0u8; 4];
    file.read_exact(&mut magic_bytes)?;

    // Try mcfg-based extraction first (real document dimensions)
    if magic_bytes == [0x50, 0x4B, 0x03, 0x04] {
        // ZIP CDR: extract internal RIFF and parse mcfg
        if let Ok(dimensions) = extract_zip_riff_dimensions(path) {
            return Ok(dimensions);
        }
    } else if magic_bytes == *b"RIFF" {
        // RIFF CDR: parse mcfg directly
        if let Ok(dimensions) = extract_riff_mcfg_dimensions(path) {
            return Ok(dimensions);
        }
    }

    // Fallback: extract from preview image header
    if let Ok((preview_data, _mime_type)) = extract_coreldraw_preview(path) {
        if let Ok(reader) =
            image::ImageReader::new(Cursor::new(&preview_data)).with_guessed_format()
        {
            if let Ok((width, height)) = reader.into_dimensions() {
                return Ok((width, height));
            }
        }
    }
    Err("Could not extract CDR dimensions".into())
}

/// Extracts technical and semantic metadata from a CorelDRAW file.
///
/// Combines version detection, document dimensions, and preview analysis.
///
/// # Arguments
///
/// * `path` - Path to the CDR file.
///
/// # Returns
///
/// `Result<serde_json::Value, Box<dyn std::error::Error>>` - JSON containing technical data.
pub fn extract_coreldraw_metadata(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let version_string = get_cdr_version_string(path).unwrap_or_else(|| "Unknown".to_string());
    let dimensions = extract_coreldraw_dimensions(path).ok();

    let mut technical_metadata = json!({
        "version": version_string,
    });

    if let Some((width, height)) = dimensions {
        technical_metadata["width"] = width.into();
        technical_metadata["height"] = height.into();
        technical_metadata["unit"] = "mm".into();
    }

    // Try to get resolution from preview if available
    if let Ok((preview_data, _)) = extract_coreldraw_preview(path) {
        if let Ok(_reader) =
            image::ImageReader::new(Cursor::new(&preview_data)).with_guessed_format()
        {
            // image crate doesn't easily expose DPI without diving into specific decoders
            // for now we'll stick to dimensions.
        }
    }

    Ok(json!({
        "technical": technical_metadata,
        "semantic": {}
    }))
}

/// Detects the CDR version from the RIFF header signature.
///
/// The 4th byte of the CDR signature (bytes 8-11 after "RIFF" + size)
/// encodes the version: '5'=v500, '6'=v600, ... 'A'=v1000, 'B'=v1100, etc.
///
/// Returns version as integer (e.g., 500, 600, 700, ..., 1800).
fn parse_cdr_version(signature: &[u8; 4]) -> u32 {
    if signature[0..3] != *b"CDR" && signature[0..3] != *b"cdr" {
        return 0;
    }
    let version_byte = signature[3];
    // Special case: "cdr8" (bidi) = v801
    if signature[0..3] == *b"cdr" && version_byte == 0x38 {
        return 801;
    }
    match version_byte {
        0x20 => 300,                                       // space = v3.0
        0x31..=0x39 => 100 * (version_byte as u32 - 0x30), // '1'-'9' = v100-v900
        0x41..=0x48 => 100 * (version_byte as u32 - 0x37), // 'A'-'H' = v1000-v1800
        _ => 0,
    }
}

/// Extracts the CDR version string for metadata display.
///
/// Converts the internal version number to a human-readable format string.
pub fn get_cdr_version_string(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut magic_bytes = [0u8; 4];
    file.read_exact(&mut magic_bytes).ok()?;

    if magic_bytes == [0x50, 0x4B, 0x03, 0x04] {
        // ZIP CDR: modern format (v16+)
        return Some("ZIP (v16+)".to_string());
    }

    if magic_bytes == *b"RIFF" {
        file.seek(SeekFrom::Start(8)).ok()?;
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature).ok()?;
        let version = parse_cdr_version(&signature);
        if version > 0 {
            return Some(format!("RIFF v{}.0", version / 100));
        }
    }

    if magic_bytes[0] == 0x57 && magic_bytes[1] == 0x4C {
        return Some("WL (v3-v5)".to_string());
    }

    None
}

/// Extracts document dimensions from the `mcfg` chunk in a RIFF CDR file.
///
/// The mcfg chunk stores real page size in CDR internal units.
/// Conversion: CDR units → mm = value / 10000.0
///
/// Offsets before page_size vary by CDR version:
/// - v≥1300: 12 bytes
/// - v≥900: 4 bytes
/// - v600-699: 0x1c bytes
/// - v<600: 0 bytes
fn extract_riff_mcfg_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut riff_header = [0u8; 12];
    file.read_exact(&mut riff_header)?;

    if &riff_header[0..4] != b"RIFF" {
        return Err("Not a RIFF file".into());
    }

    let mut signature = [0u8; 4];
    signature.copy_from_slice(&riff_header[8..12]);
    let version = parse_cdr_version(&signature);

    let file_length = file.metadata()?.len();
    let mut mcfg_data: Option<Vec<u8>> = None;
    walk_riff_for_chunk(&mut file, 12, file_length, b"mcfg", &mut mcfg_data)?;

    if let Some(data) = mcfg_data {
        return parse_mcfg_dimensions(&data, version);
    }

    Err("mcfg chunk not found".into())
}

/// Extracts document dimensions from the internal RIFF in a ZIP CDR file.
///
/// ZIP-based CDR files (v16+) contain `content/riffData.cdr` which holds
/// the complete RIFF structure with mcfg and other metadata chunks.
fn extract_zip_riff_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Try to extract the internal RIFF data
    let riff_entry_name = if archive.by_name("content/riffData.cdr").is_ok() {
        "content/riffData.cdr"
    } else if archive.by_name("content/root.dat").is_ok() {
        "content/root.dat"
    } else {
        return Err("No internal RIFF found in ZIP CDR".into());
    };

    let mut entry = archive.by_name(riff_entry_name)?;
    let mut riff_data = Vec::new();
    entry.read_to_end(&mut riff_data)?;

    // Parse the RIFF structure from the extracted data
    if riff_data.len() < 12 || &riff_data[0..4] != b"RIFF" {
        return Err("Invalid RIFF data in ZIP CDR".into());
    }

    let mut signature = [0u8; 4];
    signature.copy_from_slice(&riff_data[8..12]);
    let version = parse_cdr_version(&signature);

    let mut cursor = Cursor::new(riff_data);
    let riff_length = cursor.get_ref().len() as u64;
    let mut mcfg_data: Option<Vec<u8>> = None;
    walk_riff_for_chunk(&mut cursor, 12, riff_length, b"mcfg", &mut mcfg_data)?;

    if let Some(data) = mcfg_data {
        return parse_mcfg_dimensions(&data, version);
    }

    Err("mcfg chunk not found in ZIP CDR RIFF".into())
}

/// Walks RIFF chunks searching for a specific chunk by identifier.
///
/// Traverses LIST containers (including compressed cmpr blocks) to find
/// the target chunk. Stores the first match found.
fn walk_riff_for_chunk<R: Read + Seek>(
    reader: &mut R,
    start_position: u64,
    end_position: u64,
    target_chunk_id: &[u8; 4],
    result: &mut Option<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    reader.seek(SeekFrom::Start(start_position))?;

    while reader.stream_position()? + 8 <= end_position && result.is_none() {
        let mut chunk_identifier = [0u8; 4];
        if reader.read_exact(&mut chunk_identifier).is_err() {
            break;
        }

        let chunk_size = reader.read_u32::<LittleEndian>()?;
        if chunk_size > 100_000_000 {
            break;
        }

        let next_chunk_position =
            reader.stream_position()? + chunk_size as u64 + (chunk_size % 2) as u64;

        if &chunk_identifier == target_chunk_id {
            let mut chunk_data = vec![0u8; chunk_size as usize];
            reader.read_exact(&mut chunk_data)?;
            *result = Some(chunk_data);
            return Ok(());
        }

        if chunk_identifier == *b"LIST" {
            let mut list_type = [0u8; 4];
            reader.read_exact(&mut list_type)?;

            if list_type == *b"cmpr" && chunk_size > 40 {
                let mut compressed_data = vec![0u8; (chunk_size - 4) as usize];
                reader.read_exact(&mut compressed_data)?;
                if compressed_data.len() > 32 {
                    let zlib_offset = 24;
                    if zlib_offset < compressed_data.len() {
                        let zlib_stream = &compressed_data[zlib_offset..];
                        let mut decoder = ZlibDecoder::new(zlib_stream);
                        let mut decompressed_data = Vec::new();
                        if decoder.read_to_end(&mut decompressed_data).is_ok() {
                            let mut decompressed_cursor = Cursor::new(decompressed_data);
                            let decompressed_length = decompressed_cursor.get_ref().len() as u64;
                            let _ = walk_riff_for_chunk(
                                &mut decompressed_cursor,
                                0,
                                decompressed_length,
                                target_chunk_id,
                                result,
                            );
                        }
                    }
                }
            } else {
                let list_content_start = reader.stream_position()?;
                let _ = walk_riff_for_chunk(
                    reader,
                    list_content_start,
                    next_chunk_position,
                    target_chunk_id,
                    result,
                );
            }
        }

        reader.seek(SeekFrom::Start(next_chunk_position))?;
    }

    Ok(())
}

/// Parses the mcfg chunk data to extract page width and height.
///
/// CDR internal units are in 1/10000 mm. The offset before the page_size
/// data depends on the CDR version.
fn parse_mcfg_dimensions(
    mcfg_data: &[u8],
    version: u32,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let skip_offset: usize = if version >= 1300 {
        12
    } else if version >= 900 {
        4
    } else if version >= 600 && version < 700 {
        0x1c
    } else {
        0
    };

    if version < 400 {
        // Old format: bounding box with x0, y0, x1, y1 (each coord is 4 bytes)
        // Skip 2 bytes of unknown, then read x0, y0, x1, y1
        let data_offset = skip_offset + 2;
        if mcfg_data.len() < data_offset + 16 {
            return Err("mcfg data too short for old format".into());
        }
        let mut cursor = Cursor::new(&mcfg_data[data_offset..]);
        let x0 = cursor.read_i32::<LittleEndian>()? as f64;
        let y0 = cursor.read_i32::<LittleEndian>()? as f64;
        let x1 = cursor.read_i32::<LittleEndian>()? as f64;
        let y1 = cursor.read_i32::<LittleEndian>()? as f64;
        let width_mm = ((x1 - x0).abs() / 10000.0).round() as u32;
        let height_mm = ((y1 - y0).abs() / 10000.0).round() as u32;
        if width_mm > 0 && height_mm > 0 && width_mm < 100000 && height_mm < 100000 {
            tracing::debug!(
                "CDR: mcfg old format dimensions: {}x{} mm",
                width_mm,
                height_mm
            );
            return Ok((width_mm, height_mm));
        }
    } else {
        // New format: width, height (each coord is 4 bytes signed i32)
        if mcfg_data.len() < skip_offset + 8 {
            return Err("mcfg data too short for new format".into());
        }
        let mut cursor = Cursor::new(&mcfg_data[skip_offset..]);
        let raw_width = cursor.read_i32::<LittleEndian>()? as f64;
        let raw_height = cursor.read_i32::<LittleEndian>()? as f64;
        let width_mm = (raw_width.abs() / 10000.0).round() as u32;
        let height_mm = (raw_height.abs() / 10000.0).round() as u32;
        if width_mm > 0 && height_mm > 0 && width_mm < 100000 && height_mm < 100000 {
            tracing::debug!(
                "CDR: mcfg dimensions: {}x{} mm (v{})",
                width_mm,
                height_mm,
                version
            );
            return Ok((width_mm, height_mm));
        }
    }

    Err("Invalid mcfg dimensions".into())
}

/// Checks whether a file is a legacy (non-ZIP) format.
fn is_legacy_format(path: &Path) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut magic_bytes = [0u8; 4];
        if file.read_exact(&mut magic_bytes).is_ok() {
            return magic_bytes != [0x50, 0x4B, 0x03, 0x04];
        }
    }
    true
}

// ─── Modern ZIP Strategy ─────────────────────────────────────────────

/// Extracts the best-quality preview from a ZIP-based CDR file.
///
/// Checks known preview paths in priority order, selecting the largest
/// candidate by decompressed size. Falls back to scanning all entries
/// for PNG files matching preview/page/thumb patterns.
fn extract_zip_best_quality(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let known_paths = [
        "previews/page1.png",
        "content/preview.png",
        "previews/thumbnail.png",
    ];

    // Collect all known preview candidates and select by pixel resolution
    let mut all_candidates: Vec<(Vec<u8>, String, u32)> = Vec::new();

    for candidate_path in known_paths {
        if let Ok(mut entry) = archive.by_name(candidate_path) {
            let mut buffer = Vec::new();
            if entry.read_to_end(&mut buffer).is_ok() && buffer.len() > 100 {
                let pixel_count = get_pixel_count(&buffer);
                tracing::debug!(
                    "CDR: ZIP candidate '{}' size={} bytes, pixels={}",
                    candidate_path,
                    buffer.len(),
                    pixel_count
                );
                all_candidates.push((buffer, "image/png".to_string(), pixel_count));
            }
        }
    }

    // Fallback: scan all entries for image files
    if all_candidates.is_empty() {
        let mut candidate_indices: Vec<usize> = Vec::new();
        for entry_index in 0..archive.len() {
            if let Ok(entry) = archive.by_index(entry_index) {
                let entry_name = entry.name().to_lowercase();
                if entry_name.ends_with(".png")
                    && (entry_name.contains("preview")
                        || entry_name.contains("page")
                        || entry_name.contains("thumb"))
                {
                    candidate_indices.push(entry_index);
                }
            }
        }
        for entry_index in candidate_indices {
            if let Ok(mut entry) = archive.by_index(entry_index) {
                let mut buffer = Vec::new();
                if entry.read_to_end(&mut buffer).is_ok() && buffer.len() > 100 {
                    let pixel_count = get_pixel_count(&buffer);
                    all_candidates.push((buffer, "image/png".to_string(), pixel_count));
                }
            }
        }
    }

    // Select candidate with highest pixel count (best resolution)
    if let Some((best_data, best_mime, best_pixels)) = all_candidates
        .into_iter()
        .max_by_key(|(_, _, pixel_count)| *pixel_count)
    {
        tracing::debug!("CDR: Selected ZIP preview with {} pixels", best_pixels);
        return Ok((best_data, best_mime));
    }

    Err("No preview found in CorelDRAW ZIP container".into())
}

/// Calculates total pixel count from image data for resolution-based selection.
fn get_pixel_count(image_data: &[u8]) -> u32 {
    if let Ok(reader) = image::ImageReader::new(Cursor::new(image_data)).with_guessed_format() {
        if let Ok((width, height)) = reader.into_dimensions() {
            return width * height;
        }
    }
    0
}

// ─── Legacy RIFF Strategy ────────────────────────────────────────────

/// Extracts all preview candidates from a RIFF-based CDR file.
#[allow(clippy::type_complexity)]
fn extract_riff_previews(
    path: &Path,
) -> Result<Vec<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(8))?;
    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)?;

    if signature != *b"CDR " && signature != *b"CDRB" && signature != *b"CDRD" {
        return Err("Not a valid RIFF CDR file".into());
    }

    let file_length = file.metadata()?.len();
    let mut candidates = Vec::new();
    walk_riff_generic(&mut file, 12, file_length, &mut candidates)?;
    Ok(candidates)
}

/// Recursively walks RIFF chunks searching for embedded preview images.
///
/// Handles LIST containers (including compressed `cmpr` blocks via zlib),
/// and extracts image data from DISP, icp0, bmp, and imhd chunks.
fn walk_riff_generic<R: Read + Seek>(
    reader: &mut R,
    start_position: u64,
    end_position: u64,
    candidates: &mut Vec<(Vec<u8>, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    reader.seek(SeekFrom::Start(start_position))?;

    while reader.stream_position()? + 8 <= end_position {
        let mut chunk_identifier = [0u8; 4];
        if reader.read_exact(&mut chunk_identifier).is_err() {
            break;
        }

        let chunk_size = reader.read_u32::<LittleEndian>()?;
        if chunk_size > 100_000_000 {
            break;
        }

        let next_chunk_position =
            reader.stream_position()? + chunk_size as u64 + (chunk_size % 2) as u64;

        if chunk_identifier == *b"LIST" {
            let list_type_position = reader.stream_position()?;
            let mut list_type = [0u8; 4];
            reader.read_exact(&mut list_type)?;

            if list_type == *b"disp"
                || list_type == *b"page"
                || list_type == *b"INFO"
                || list_type == *b"CMPR"
                || list_type == *b"doc "
                || list_type == *b"gobj"
                || list_type == *b"iccp"
            {
                match walk_riff_generic(
                    reader,
                    list_type_position + 4,
                    next_chunk_position,
                    candidates,
                ) {
                    Ok(_) => {}
                    Err(_) => {
                        reader.seek(SeekFrom::Start(next_chunk_position))?;
                    }
                }
            } else if list_type == *b"cmpr" && chunk_size > 40 {
                let mut compressed_data = vec![0u8; (chunk_size - 4) as usize];
                reader.read_exact(&mut compressed_data)?;

                if compressed_data.len() > 32 {
                    let zlib_offset = 24;
                    if zlib_offset < compressed_data.len() {
                        let zlib_stream = &compressed_data[zlib_offset..];
                        let mut decoder = ZlibDecoder::new(zlib_stream);
                        let mut decompressed_data = Vec::new();
                        if decoder.read_to_end(&mut decompressed_data).is_ok() {
                            let mut cursor = Cursor::new(decompressed_data);
                            let decompressed_length = cursor.get_ref().len() as u64;
                            let _ =
                                walk_riff_generic(&mut cursor, 0, decompressed_length, candidates);
                        }
                    }
                }
                reader.seek(SeekFrom::Start(next_chunk_position))?;
            } else {
                reader.seek(SeekFrom::Start(next_chunk_position))?;
            }
        } else if &chunk_identifier == b"DISP"
            || &chunk_identifier == b"icp0"
            || &chunk_identifier == b"bmp "
            || &chunk_identifier == b"imhd"
        {
            if chunk_size == 0 {
                reader.seek(SeekFrom::Start(next_chunk_position))?;
                continue;
            }

            let mut chunk_data = vec![0u8; chunk_size as usize];
            reader.read_exact(&mut chunk_data)?;

            if !check_and_extract_image(&chunk_data, candidates) {
                // BMP variant 1: starts with "BM"
                if chunk_data.len() > 2 && chunk_data[0] == b'B' && chunk_data[1] == b'M' {
                    candidates.push((chunk_data.clone(), "image/bmp".to_string()));
                }
                // BMP variant 2: "BM" at offset 4 (4-byte prefix)
                else if chunk_data.len() > 6 && chunk_data[4] == b'B' && chunk_data[5] == b'M' {
                    candidates.push((chunk_data[4..].to_vec(), "image/bmp".to_string()));
                }
                // DIB variant 1: BITMAPINFOHEADER (0x28) at offset 4
                else if chunk_data.len() > 8
                    && chunk_data[4] == 0x28
                    && chunk_data[5] == 0x00
                    && chunk_data[6] == 0x00
                    && chunk_data[7] == 0x00
                {
                    if let Ok(bmp_data) = construct_bmp_from_dib(&chunk_data[4..]) {
                        candidates.push((bmp_data, "image/bmp".to_string()));
                    }
                }
                // DIB variant 2: BITMAPINFOHEADER (0x28) at offset 1
                else if chunk_data.len() > 5
                    && chunk_data[1] == 0x28
                    && chunk_data[2] == 0x00
                    && chunk_data[3] == 0x00
                {
                    if let Ok(bmp_data) = construct_bmp_from_dib(&chunk_data[1..]) {
                        candidates.push((bmp_data, "image/bmp".to_string()));
                    }
                }
                // DIB variant 3: BITMAPINFOHEADER (0x28) at offset 0
                else if chunk_data.len() > 4
                    && chunk_data[0] == 0x28
                    && chunk_data[1] == 0x00
                    && chunk_data[2] == 0x00
                {
                    if let Ok(bmp_data) = construct_bmp_from_dib(&chunk_data) {
                        candidates.push((bmp_data, "image/bmp".to_string()));
                    }
                }
            }

            reader.seek(SeekFrom::Start(next_chunk_position))?;
        } else {
            reader.seek(SeekFrom::Start(next_chunk_position))?;
        }
    }
    Ok(())
}

/// Detects and extracts known image formats (PNG, JPEG, TIFF, GIF)
/// embedded inside RIFF chunk data.
///
/// Returns `true` if a recognized image was found and added to candidates.
fn check_and_extract_image(data: &[u8], candidates: &mut Vec<(Vec<u8>, String)>) -> bool {
    // PNG: 89 50 4E 47
    if data.len() > 8 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        candidates.push((data.to_vec(), "image/png".to_string()));
        return true;
    }
    // JPEG: FF D8 FF
    if data.len() > 2 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        candidates.push((data.to_vec(), "image/jpeg".to_string()));
        return true;
    }
    // TIFF: II* (little-endian) or MM* (big-endian)
    if data.len() > 4
        && ((data[0] == 0x49 && data[1] == 0x49 && data[2] == 0x2A && data[3] == 0x00)
            || (data[0] == 0x4D && data[1] == 0x4D && data[2] == 0x00 && data[3] == 0x2A))
    {
        candidates.push((data.to_vec(), "image/tiff".to_string()));
        return true;
    }
    // GIF: GIF8
    if data.len() > 3 && data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
        candidates.push((data.to_vec(), "image/gif".to_string()));
        return true;
    }
    false
}

/// Constructs a full BMP file from a raw DIB (Device Independent Bitmap) block.
///
/// Prepends the 14-byte BMP file header with correct pixel data offset,
/// accounting for palette entries when bit depth <= 8.
///
/// # Errors
/// Returns error if DIB data is too short or header size is invalid.
fn construct_bmp_from_dib(dib_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if dib_data.len() < 4 {
        return Err("Too short".into());
    }

    let header_size = u32::from_le_bytes([dib_data[0], dib_data[1], dib_data[2], dib_data[3]]);
    if dib_data.len() < header_size as usize {
        return Err("DIB smaller than header".into());
    }

    let dib_length = dib_data.len() as u32;
    let file_size = 14 + dib_length;

    let mut palette_size: u32 = 0;
    if header_size >= 16 && dib_data.len() >= 16 {
        let bits_per_pixel = u16::from_le_bytes([dib_data[14], dib_data[15]]);
        if bits_per_pixel <= 8 {
            let mut colors_used: u32 = 0;
            if header_size >= 36 && dib_data.len() >= 36 {
                colors_used =
                    u32::from_le_bytes([dib_data[32], dib_data[33], dib_data[34], dib_data[35]]);
            }
            palette_size = if colors_used > 0 {
                colors_used * 4
            } else {
                (1u32 << bits_per_pixel) * 4
            };
        }
    }

    let pixel_offset = 14 + header_size + palette_size;

    let mut bmp_data = Vec::with_capacity(file_size as usize);
    bmp_data.write_all(b"BM")?;
    bmp_data.write_all(&file_size.to_le_bytes())?;
    bmp_data.write_all(&[0u8; 4])?;
    bmp_data.write_all(&pixel_offset.to_le_bytes())?;
    bmp_data.write_all(dib_data)?;

    Ok(bmp_data)
}

// ─── WL Legacy Strategy (CDR v3–v5) ─────────────────────────────────

/// Extracts the 1-bit monochrome thumbnail from a WL-signature CDR file.
///
/// These ancient CorelDRAW files (v3–v5) store a small bitmap at a fixed
/// offset (0x56) with dimensions at offsets 0x48–0x4B.
fn extract_wl_thumbnail(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header_buffer = [0u8; 1024];
    file.read_exact(&mut header_buffer)?;

    let width = u16::from_be_bytes([header_buffer[0x48], header_buffer[0x49]]) as u32;
    let height = u16::from_be_bytes([header_buffer[0x4A], header_buffer[0x4B]]) as u32;

    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err("Invalid WL header dimensions".into());
    }

    tracing::debug!("CDR: WL Thumbnail detected: {}x{}", width, height);

    let stride = width.div_ceil(32) * 4;
    let data_size = (stride * height) as usize;
    let start_offset: usize = 0x56;

    if start_offset + data_size > header_buffer.len() {
        file.seek(SeekFrom::Start(0))?;
        let mut full_data = Vec::new();
        file.read_to_end(&mut full_data)?;
        if start_offset + data_size > full_data.len() {
            return Err("WL data too short".into());
        }
        let raw_bits = &full_data[start_offset..start_offset + data_size];
        return wrap_raw_1bit_bmp(raw_bits, width, height);
    }
    let raw_bits = &header_buffer[start_offset..start_offset + data_size];
    wrap_raw_1bit_bmp(raw_bits, width, height)
}

/// Wraps raw 1-bit pixel data into a valid BMP file with a 2-color palette.
fn wrap_raw_1bit_bmp(
    raw_data: &[u8],
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file_header_size: u32 = 14;
    let info_header_size: u32 = 40;
    let palette_size: u32 = 2 * 4;
    let pixel_offset = file_header_size + info_header_size + palette_size;
    let file_size = pixel_offset + raw_data.len() as u32;

    let mut bmp_data = Vec::with_capacity(file_size as usize);
    bmp_data.write_all(b"BM")?;
    bmp_data.write_all(&file_size.to_le_bytes())?;
    bmp_data.write_all(&[0u8; 4])?;
    bmp_data.write_all(&pixel_offset.to_le_bytes())?;
    bmp_data.write_all(&info_header_size.to_le_bytes())?;
    bmp_data.write_all(&(width as i32).to_le_bytes())?;
    bmp_data.write_all(&(-(height as i32)).to_le_bytes())?;
    bmp_data.write_all(&(1u16).to_le_bytes())?; // Planes
    bmp_data.write_all(&(1u16).to_le_bytes())?; // BPP
    bmp_data.write_all(&[0u8; 4])?; // Compression
    bmp_data.write_all(&(raw_data.len() as u32).to_le_bytes())?;
    bmp_data.write_all(&[0u8; 16])?;
    bmp_data.write_all(&[0x00, 0x00, 0x00, 0x00])?; // Palette color 0: black
    bmp_data.write_all(&[0xFF, 0xFF, 0xFF, 0x00])?; // Palette color 1: white
    bmp_data.write_all(raw_data)?;

    Ok((bmp_data, "image/bmp".to_string()))
}

// ─── Fallback: Embedded Image Scan ───────────────────────────────────

/// Scans the entire file binary for embedded BMP and PNG images.
///
/// This is a brute-force fallback for legacy or unusual CDR files where
/// structured RIFF parsing fails. JPEG scanning is deliberately omitted
/// because 0xFF 0xD8 produces too many false positives in binary data.
#[allow(clippy::type_complexity)]
fn scan_for_embedded_images(
    path: &Path,
) -> Result<Vec<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut found: Vec<(Vec<u8>, String)> = Vec::new();
    let total_length = buffer.len();
    let mut scan_position: usize = 0;

    while scan_position < total_length.saturating_sub(16) {
        // BMP: 'BM' + declared size validation
        if buffer[scan_position] == 0x42 && buffer[scan_position + 1] == 0x4D {
            let size_bytes = &buffer[scan_position + 2..scan_position + 6];
            if let Ok(size_array) = <[u8; 4]>::try_from(size_bytes) {
                let declared_size = u32::from_le_bytes(size_array) as usize;
                if declared_size > 50 && (scan_position + declared_size) <= total_length {
                    if let Ok(offset_array) =
                        <[u8; 4]>::try_from(&buffer[scan_position + 10..scan_position + 14])
                    {
                        let pixel_offset = u32::from_le_bytes(offset_array) as usize;
                        if (14..10000).contains(&pixel_offset) {
                            let bmp_data =
                                buffer[scan_position..scan_position + declared_size].to_vec();
                            found.push((bmp_data, "image/bmp".to_string()));
                            scan_position += declared_size;
                            continue;
                        }
                    }
                }
            }
        }

        // PNG: strict 8-byte magic check + IEND scan
        if scan_position + 8 <= total_length
            && buffer[scan_position] == 0x89
            && buffer[scan_position + 1] == 0x50
            && buffer[scan_position + 2] == 0x4E
            && buffer[scan_position + 3] == 0x47
            && buffer[scan_position + 4] == 0x0D
            && buffer[scan_position + 5] == 0x0A
            && buffer[scan_position + 6] == 0x1A
            && buffer[scan_position + 7] == 0x0A
        {
            let mut iend_position: usize = 0;
            for search_position in (scan_position + 8)..total_length.saturating_sub(8) {
                if buffer[search_position] == 0x49      // I
                    && buffer[search_position + 1] == 0x45  // E
                    && buffer[search_position + 2] == 0x4E  // N
                    && buffer[search_position + 3] == 0x44
                // D
                {
                    iend_position = search_position + 8; // IEND tag(4) + CRC(4)
                    break;
                }
            }
            if iend_position > 0 && iend_position <= total_length {
                let png_data = buffer[scan_position..iend_position].to_vec();
                if png_data.len() > 60 {
                    tracing::debug!(
                        "CDR: Found embedded PNG at offset {}. Size: {}",
                        scan_position,
                        png_data.len()
                    );
                    found.push((png_data, "image/png".to_string()));
                    scan_position = iend_position;
                    continue;
                }
            }
        }

        scan_position += 1;
    }

    if found.is_empty() {
        return Err("No embedded images found".into());
    }
    Ok(found)
}
