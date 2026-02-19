use std::fs::File;
use std::io::Read;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use crate::thumbnails::extractors::binary_jpeg;

/// Main entry point for AI file preview extraction.
/// Implements a hybrid strategy: XMP -> PDF -> Binary Fallback.
pub fn extract_ai_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    println!("AI_EXTRACT: Analyzing {:?}", path);
    // 1. Try XMP Metadata Thumbnail (Fastest, usually JPEG)
    // We implement a binary-safe scan here instead of relying on string conversion
    // because .ai files are binary and often fail UTF-8 validation.
    if let Ok(data) = extract_xmp_thumbnail_safe(path) {
        println!("AI_EXTRACT: Found XMP thumbnail ({} bytes)", data.len());
        return Ok((data, "image/jpeg".to_string()));
    } else {
        println!("AI_EXTRACT: XMP extraction failed");
    }

    // 2. Try PDF Stream (Highest Quality)
    // This allows the PDF backend to render the vector content.
    if let Ok(data) = extract_ai_pdf_stream(path) {
        println!("AI_EXTRACT: Found PDF stream ({} bytes)", data.len());
        return Ok((data, "application/pdf".to_string()));
    } else {
        println!("AI_EXTRACT: PDF stream extraction failed");
    }

    // 3. Fallback to binary scanner (Legacy AI / PDF-incompatible)
    // This finds any embedded JPEG/TIFF/PNG using purely binary signatures.
    if let Ok((data, mime)) = binary_jpeg::extract_any_embedded(path) {
        println!("AI_EXTRACT: Found embedded {} ({} bytes)", mime, data.len());
        // If we found a TIFF, we might want to let the caller handle it (convert to PNG if needed),
        // but here we just return the raw data and let the pipeline decide.
        // In mod.rs, there is logic to convert TIFF to PNG if needed.
        return Ok((data, mime));
    } else {
        println!("AI_EXTRACT: Binary scanning failed");
    }

    Err("No preview found in AI file".into())
}

/// Binary-safe extraction of XMP thumbnail.
/// Searches for <xmpGImg:image> tags without assuming the file is valid UTF-8 text.
fn extract_xmp_thumbnail_safe(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut buffer = Vec::new();

    // Read up to 10MB to find metadata. XMP is usually in the header or early body.
    // We don't read the whole file to avoid loading massive AI files into RAM just for a check.
    file.take(10 * 1024 * 1024).read_to_end(&mut buffer)?;

    // Optimization: Find the rough location of XMP block first using byte search
    let start_marker = b"<xmpGImg:image>";
    let end_marker = b"</xmpGImg:image>";

    if let Some(start_pos) = buffer.windows(start_marker.len()).position(|w| w == start_marker) {
        let content_start = start_pos + start_marker.len();

        if let Some(end_rel) = buffer[content_start..].windows(end_marker.len()).position(|w| w == end_marker) {
            let content_end = content_start + end_rel;
            let raw_bytes = &buffer[content_start..content_end];

            // Convert to string (lossy is fine, base64 is ASCII) to handle XML entities
            let raw_str = String::from_utf8_lossy(raw_bytes);

            // Sanitize:
            // 1. Remove XML newlines (&#xA;)
            // 2. Remove actual whitespace/newlines
            // 3. Keep only valid Base64 chars
            let clean_str: String = raw_str
                .replace("&#xA;", "")
                .replace("&#xD;", "")
                .chars()
                .filter(|c| {
                    c.is_alphanumeric() || *c == '+' || *c == '/' || *c == '='
                })
                .collect();

            // Decode
            let decoded = general_purpose::STANDARD.decode(clean_str)?;
            return Ok(decoded);
        }
    }

    Err("No XMP thumbnail tags found".into())
}

/// Extracts the PDF compatibility stream from the AI file.
/// AI files with "Create PDF Compatible File" checked are valid PDF files.
pub fn extract_ai_pdf_stream(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Check for PDF signatures
    if let Some(start) = buffer.windows(5).position(|w| w == b"%PDF-") {
        if let Some(end_rel) = buffer[start..].windows(5).rposition(|w| w == b"%%EOF") {
            let end = start + end_rel + 5;
            // Return the PDF slice
            return Ok(buffer[start..end].to_vec());
        }
        // If no EOF found (rare/truncated), try returning from start to end
        return Ok(buffer[start..].to_vec());
    }

    Err("Not a PDF-compatible AI file".into())
}
