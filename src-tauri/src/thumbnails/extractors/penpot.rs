use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Extracts a preview (thumbnail) from a Penpot project file (.penpot).
/// Handles both V1 (ZIP container) and V2 (Zstd compressed stream).
///
/// # Arguments
/// * `path` - The path to the Penpot file.
///
/// # Errors
/// Returns `Err` if the file cannot be read, format is unsupported, or no preview image is found.
pub fn extract_penpot_preview(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];

    // Read the first 4 bytes cautiously
    if file.read(&mut magic)? < 4 {
        return Err("File too small to have a valid header".into());
    }

    // V1 (ZIP Archive)
    if magic == [0x50, 0x4B, 0x03, 0x04] {
        return extract_v1_zip_preview(&mut file);
    }

    // V2 (Zstd Binary)
    // Check first 4 bytes of magic header: `01 0B 1A 86`
    if magic == [0x01, 0x0B, 0x1A, 0x86] {
        return extract_v2_zstd_preview(path);
    }

    Err(format!("Unknown Penpot format magic bytes: {:?}", magic).into())
}

/// Extracts the largest PNG found in the `objects/` directory of a V1 structure.
fn extract_v1_zip_preview(file: &mut File) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(0))?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut largest_size = 0;
    let mut best_index = None;

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            // Thumbnails in V1 are stored inside `objects/` with `.png`.
            if name.starts_with("objects/") && name.ends_with(".png") {
                let size = entry.size();
                if size > largest_size {
                    largest_size = size;
                    best_index = Some(i);
                }
            }
        }
    }

    if let Some(index) = best_index {
        let mut entry = archive.by_index(index)?;
        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer)?;
        return Ok(buffer);
    }

    Err("No valid PNG found in objects directory of Penpot V1 file".into())
}

/// Decompresses the V2 Zstandard stream and extracts the largest embedded PNG.
fn extract_v2_zstd_preview(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    // The Zstd payload starts at offset 17
    file.seek(SeekFrom::Start(17))?;

    let decoder = zstd::stream::Decoder::new(file)?;
    // Use `take` to securely limit the decompressed bytes (protects memory if files are giant)
    let max_read_limit = 50 * 1024 * 1024; // 50 MB limit
    let mut limited_reader = decoder.take(max_read_limit);

    let mut buffer = Vec::new();
    let _ = limited_reader.read_to_end(&mut buffer);

    if let Some(png_data) = extract_largest_png_from_buffer(&buffer) {
        return Ok(png_data);
    }

    Err("No valid PNG chunks could be extracted from decompressed V2 stream".into())
}

/// Helper function to perform a manual binary scan for all PNG markers, calculate size with chunks, and return the largest.
fn extract_largest_png_from_buffer(buffer: &[u8]) -> Option<Vec<u8>> {
    let png_magic = b"\x89PNG\r\n\x1a\n";
    let mut largest_png = None;
    let mut largest_size = 0;

    let mut offset = 0;
    while offset < buffer.len().saturating_sub(png_magic.len()) {
        if let Some(mut start) = find_subsequence(&buffer[offset..], png_magic) {
            start += offset;
            offset = start + png_magic.len(); // Move past magic

            let mut chunk_offset = offset;
            let mut found_iend = false;

            // Parse PNG chunks sequentially until IEND is met
            while chunk_offset + 8 <= buffer.len() {
                // Length (4 bytes, Big Endian)
                let length = u32::from_be_bytes([
                    buffer[chunk_offset],
                    buffer[chunk_offset + 1],
                    buffer[chunk_offset + 2],
                    buffer[chunk_offset + 3],
                ]) as usize;

                let chunk_type = &buffer[chunk_offset + 4..chunk_offset + 8];

                // Advance past Length (4), Type (4), Data (length), CRC (4)
                chunk_offset += 8 + length + 4;

                if chunk_offset > buffer.len() {
                    break; // Corrupted PNG boundary or buffer was truncated
                }

                if chunk_type == b"IEND" {
                    found_iend = true;
                    break;
                }
            }

            if found_iend {
                let png_size = chunk_offset - start;
                if png_size > largest_size {
                    largest_size = png_size;
                    largest_png = Some(buffer[start..chunk_offset].to_vec());
                }
                offset = chunk_offset; // Skip the PNG we just completely mapped
            }
        } else {
            break; // No more signatures found
        }
    }

    largest_png
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
