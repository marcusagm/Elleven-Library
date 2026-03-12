//! Adobe Illustrator (.ai) preview extractor.
//!
//! Ported from V1 backend.

use crate::processing::media::extractors::binary_jpeg;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn extract_ai_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    if let Ok(pdf) = extract_ai_pdf_stream(path) {
        return Ok((pdf, "application/pdf".to_string()));
    }
    if let Ok(data) = binary_jpeg::extract_xmp_thumbnail(path) {
        return Ok((data, "image/png".to_string()));
    }
    if let Ok(res) = binary_jpeg::extract_any_embedded(path) {
        return Ok(res);
    }
    Err("No preview found in AI file".into())
}

pub fn extract_ai_pdf_stream(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    if let Some(start) = buf.windows(5).position(|w| w == b"%PDF-") {
        if let Some(end_rel) = buf[start..].windows(5).rposition(|w| w == b"%%EOF") {
            let end = start + end_rel + 5;
            return Ok(buf[start..end].to_vec());
        }
        return Ok(buf[start..].to_vec());
    }
    Err("Not a PDF-compatible AI file".into())
}
