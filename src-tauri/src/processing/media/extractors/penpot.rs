//! Penpot project (.penpot) preview extractor.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Maximum size for decompressed Zstd stream to prevent memory exhaustion.
const MAX_DECOMPRESSION_LIMIT_BYTES: u64 = 50 * 1024 * 1024; // 50 MB

/// Extracts the preview from a Penpot file.
///
/// # Penpot File Format
///
/// Penpot V1: .penpot files are ZIP archives containing:
/// - `manifest.json` - Metadata about the project (width, height, etc.)
/// - `objects/` - Directory containing PNG previews of layers
///
/// Penpot V2: .penpot files are Zstd compressed binary files containing:
/// - Binary data with Zstd compression
/// - Preview image embedded in the compressed data
///
/// # Arguments
///
/// * `path` - Path to the Penpot file
///
/// # Returns
///
/// Returns a tuple containing the preview image data and its MIME type.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::extractors::penpot::extract_penpot_preview;
/// use std::path::Path;
///
/// let path = Path::new("test.penpot");
/// let (preview_data, mime_type) = extract_penpot_preview(path).unwrap();
///
/// assert_eq!(mime_type, "image/png");
/// assert!(!preview_data.is_empty());
/// ```
pub fn extract_penpot_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic_header = [0u8; 4];
    if file.read(&mut magic_header)? < 4 {
        return Err("File too small to have a valid Penpot header".into());
    }

    // V1 (ZIP Archive)
    if magic_header == [0x50, 0x4B, 0x03, 0x04] {
        return extract_v1_zip_preview(&mut file);
    }
    // V2 (Zstd Binary)
    if magic_header == [0x01, 0x0B, 0x1A, 0x86] {
        return extract_v2_zstd_preview(path);
    }
    Err("Unknown or unsupported Penpot format version".into())
}

/// Extract Penpot V1 Preview (ZIP)
///
/// # Arguments
///
/// * `file` - A mutable reference to a file opened at the beginning of the Penpot V1 file (ZIP format).
///
/// # Returns
///
/// Returns a tuple containing the preview image data and its MIME type.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::extractors::penpot::extract_v1_zip_preview;
/// use std::fs::File;
/// use std::path::Path;
///
/// let path = Path::new("test.penpot");
/// let file = File::open(path).unwrap();
/// let (preview_data, mime_type) = extract_v1_zip_preview(&file).unwrap();
///
/// assert_eq!(mime_type, "image/png");
/// assert!(!preview_data.is_empty());
/// ```
fn extract_v1_zip_preview(
    file: &mut File,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(0))?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut best_entry_index = None;
    let mut maximum_detected_size = 0;

    for index in 0..archive.len() {
        if let Ok(entry) = archive.by_index(index) {
            let entry_name = entry.name().to_lowercase();
            // Thumbnails in V1 are stored inside `objects/` with `.png`.
            if entry_name.starts_with("objects/")
                && entry_name.ends_with(".png")
                && entry.size() > maximum_detected_size
            {
                maximum_detected_size = entry.size();
                best_entry_index = Some(index);
            }
        }
    }

    if let Some(index) = best_entry_index {
        let mut entry = archive.by_index(index)?;
        let mut image_buffer = Vec::new();
        entry.read_to_end(&mut image_buffer)?;
        return Ok((image_buffer, "image/png".to_string()));
    }
    Err("No valid preview found in Penpot ZIP container".into())
}

/// Extract Penpot V2 Preview (ZSTD)
///
/// # Arguments
///
/// * `path` - Path to the Penpot file
///
/// # Returns
///
/// Returns a tuple containing the preview image data and its MIME type.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::extractors::penpot::extract_v2_zstd_preview;
/// use std::path::Path;
///
/// let path = Path::new("test.penpot");
/// let (preview_data, mime_type) = extract_v2_zstd_preview(path).unwrap();
///
/// assert_eq!(mime_type, "image/png");
/// assert!(!preview_data.is_empty());
/// ``
fn extract_v2_zstd_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    // The Zstd payload in V2 starts at offset 17
    file.seek(SeekFrom::Start(17))?;

    let decoder = zstd::stream::Decoder::new(file)?;
    let mut decompressed_buffer = Vec::new();
    decoder
        .take(MAX_DECOMPRESSION_LIMIT_BYTES)
        .read_to_end(&mut decompressed_buffer)?;

    if let Some(png_data) = scan_for_largest_png(&decompressed_buffer) {
        return Ok((png_data, "image/png".to_string()));
    }
    Err("No valid PNG preview found in decompressed Penpot Zstd stream".into())
}

/// Scan for the largest PNG in a buffer.
///
/// # Arguments
///
/// * `buffer` - The buffer to scan for PNG data.
///
/// # Returns
///
/// Returns an `Option` containing the largest PNG data found in the buffer.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::extractors::penpot::scan_for_largest_png;
///
/// let buffer = b"\x89PNG\r\n\x1a\n...PNG data...\x49\x45\x4E\x44";
/// let png_data = scan_for_largest_png(buffer);
///
/// assert!(png_data.is_some());
/// ``
fn scan_for_largest_png(buffer: &[u8]) -> Option<Vec<u8>> {
    let png_magic_bytes = b"\x89PNG\r\n\x1a\n";
    let mut best_png_data = None;
    let mut maximum_png_size = 0;
    let mut current_offset = 0;

    while current_offset < buffer.len().saturating_sub(8) {
        if let Some(position) = buffer[current_offset..]
            .windows(8)
            .position(|window| window == png_magic_bytes)
        {
            let start_position = current_offset + position;
            let mut cursor_position = start_position + 8;
            let mut found_iend_chunk = false;

            while cursor_position + 8 <= buffer.len() {
                let chunk_length = u32::from_be_bytes([
                    buffer[cursor_position],
                    buffer[cursor_position + 1],
                    buffer[cursor_position + 2],
                    buffer[cursor_position + 3],
                ]) as usize;
                let chunk_type = &buffer[cursor_position + 4..cursor_position + 8];

                // Move past Length(4) + Type(4) + Data(chunk_length) + CRC(4)
                cursor_position += 12 + chunk_length;

                if chunk_type == b"IEND" {
                    found_iend_chunk = true;
                    break;
                }
                if cursor_position > buffer.len() {
                    break;
                }
            }

            if found_iend_chunk {
                let total_png_size = cursor_position - start_position;
                if total_png_size > maximum_png_size {
                    maximum_png_size = total_png_size;
                    best_png_data = Some(buffer[start_position..cursor_position].to_vec());
                }
            }
            current_offset = start_position + 8;
        } else {
            break;
        }
    }
    best_png_data
}

/// Extracts metadata from a Penpot file.
///
/// # Arguments
///
/// * `path` - Path to the Penpot file
///
/// # Returns
///
/// Returns a JSON value containing the metadata.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::extractors::penpot::extract_penpot_metadata;
/// use std::path::Path;
///
/// let path = Path::new("test.penpot");
/// let metadata = extract_penpot_metadata(path).unwrap();
///
/// println!("Technical Metadata: {}", metadata["technical"]);
/// println!("Semantic Metadata: {}", metadata["semantic"]);
/// ``
pub fn extract_penpot_metadata(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut technical_metadata = serde_json::json!({
        "container": "Unknown",
        "metadata_support": "Limited"
    });

    let semantic_metadata = serde_json::json!({});

    let mut file = File::open(path)?;
    let mut header = [0u8; 4];
    if file.read(&mut header)? >= 4 {
        // Only ZIP V1 has manifest.json easily accessible for dimensions
        if header == [0x50, 0x4B, 0x03, 0x04] {
            technical_metadata["container"] = serde_json::json!("ZIP (Penpot V1)");
            file.seek(SeekFrom::Start(0))?;

            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                if let Ok(mut manifest_entry) = archive.by_name("manifest.json") {
                    let mut manifest_content = String::new();
                    if manifest_entry.read_to_string(&mut manifest_content).is_ok() {
                        if let Ok(manifest_json) =
                            serde_json::from_str::<serde_json::Value>(&manifest_content)
                        {
                            let width = manifest_json["width"].as_f64().unwrap_or(0.0) as u32;
                            let height = manifest_json["height"].as_f64().unwrap_or(0.0) as u32;

                            technical_metadata["width"] = serde_json::json!(width);
                            technical_metadata["height"] = serde_json::json!(height);
                            technical_metadata["metadata_source"] =
                                serde_json::json!("manifest.json");
                        }
                    }
                }
            }
        } else if header == [0x01, 0x0B, 0x1A, 0x86] {
            technical_metadata["container"] = serde_json::json!("Zstd (Penpot V2)");
            technical_metadata["metadata_support"] = serde_json::json!("Thumbnail Only");

            // Try to extract dimensions from the preview
            if let Ok((preview_data, _)) = extract_v2_zstd_preview(path) {
                if let Ok(reader) = image::ImageReader::new(std::io::Cursor::new(&preview_data))
                    .with_guessed_format()
                {
                    if let Ok((width, height)) = reader.into_dimensions() {
                        technical_metadata["width"] = serde_json::json!(width);
                        technical_metadata["height"] = serde_json::json!(height);
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({
        "technical": technical_metadata,
        "semantic": semantic_metadata,
    }))
}
