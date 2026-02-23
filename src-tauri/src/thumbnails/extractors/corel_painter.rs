use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Extracts the embedded JPEG preview from a Corel Painter (.rif/.riff) file.
///
/// Strategy:
/// 1. Validates the file header (Version 2).
/// 2. Scans the binary content for a standard JPEG signature (FF D8 FF E0).
/// 3. Extracts the data until the JPEG EOI marker (FF D9).
pub fn extract_corel_painter_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 8];

    // Attempt to read header
    if file.read_exact(&mut header).is_err() {
        return Err("File too small".into());
    }

    // Validate Version (Bytes 0-1 must be 0x00 0x02 for modern Painter files)
    // The spec says: 0x00: u16 Version (Always 0x0002 in modern files).
    let is_modern_version = header[0] == 0x00 && header[1] == 0x02;

    // If not version 2, we log a warning but might still proceed with scan if we want to be robust,
    // but the recommendation was to "Validate Header".
    // If stricly strictly following "Option A" recommendation: "Validate header... before scanning".
    if !is_modern_version {
        // Check for legacy "RIFF" signature just in case
        if &header[0..4] != b"RIFF" {
            return Err("Invalid Corel Painter header (not Version 2 or RIFF)".into());
        }
    }

    // Read file content for scanning
    // NOTE: For extremely large files, memory mapping would be better, but typical RIFs are <100MB.
    // We read into a buffer to perform the scan.
    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buffer)?;

    // JPEG Start of Image (SOI) + APP0 marker commonly found in embedded thumbnails
    let jpeg_start_sig = [0xFF, 0xD8, 0xFF, 0xE0];

    if let Some(start_offset) = find_sequence(&buffer, &jpeg_start_sig) {
        // JPEG End of Image (EOI)
        let jpeg_end_sig = [0xFF, 0xD9];

        // Search for EOI starting from the SOI
        if let Some(end_relative) = find_sequence(&buffer[start_offset..], &jpeg_end_sig) {
            let end_offset = start_offset + end_relative + 2; // Include the marker itself

            let jpeg_data = buffer[start_offset..end_offset].to_vec();
            return Ok((jpeg_data, "image/jpeg".to_string()));
        }
    }

    Err("No embedded JPEG found in Corel Painter file".into())
}

/// Simple O(N) search for a byte sequence.
fn find_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_corel_painter_preview() {
        // Path to a sample file
        let path = PathBuf::from("../file-samples/Imagens/Design/Corel Painter/Line Sketches1.rif");

        // This test only runs if the sample file exists locally (which it should in this environment)
        if path.exists() {
            let result = extract_corel_painter_preview(&path);
            assert!(
                result.is_ok(),
                "Failed to extract preview from existing file: {:?}",
                result.err()
            );

            let (data, mime) = result.unwrap();
            assert_eq!(mime, "image/jpeg");
            assert!(!data.is_empty(), "Extracted data is empty");

            // Check for JPEG header
            assert_eq!(&data[0..2], &[0xFF, 0xD8]);
        } else {
            eprintln!(
                "Skipping test_extract_corel_painter_preview: Sample file not found at {:?}",
                path
            );
        }
    }
}
