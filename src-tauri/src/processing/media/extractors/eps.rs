//! Encapsulated PostScript (.eps) preview extractor.
//!
//! Ported from V1 backend.

use crate::processing::media::extractors::{ai, binary_jpeg};
use std::path::Path;

pub fn extract_eps_ps_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    if let Ok(res) = binary_jpeg::extract_eps_binary_pointer(path) {
        return Ok(res);
    }
    if let Ok(pdf) = convert_ps_to_pdf(path) {
        return Ok((pdf, "application/pdf".to_string()));
    }
    if let Ok(data) = binary_jpeg::extract_xmp_thumbnail(path) {
        return Ok((data, "image/png".to_string()));
    }
    if let Ok(res) = binary_jpeg::extract_any_embedded(path) {
        return Ok(res);
    }
    if let Ok(pdf) = ai::extract_ai_pdf_stream(path) {
        return Ok((pdf, "application/pdf".to_string()));
    }
    Err("No preview found in EPS/PS file".into())
}

fn convert_ps_to_pdf(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let temp_db = std::env::temp_dir().join(format!("mundam_eps_{}.pdf", uuid::Uuid::new_v4()));
    let mut success = false;
    if cfg!(target_os = "macos") {
        if let Ok(out) = std::process::Command::new("pstopdf").arg(path).arg("-o").arg(&temp_db).output() {
            if out.status.success() { success = true; }
        }
    }
    if !success {
        let cmds = if cfg!(target_os = "windows") { vec!["gswin64c", "gswin32c", "gs"] } else { vec!["gs"] };
        for cmd in cmds {
            if let Ok(out) = std::process::Command::new(cmd)
                .args(["-sDEVICE=pdfwrite", "-dSAFER", "-dBATCH", "-dNOPAUSE"])
                .arg(format!("-sOutputFile={}", temp_db.display()))
                .arg(path).output() {
                if out.status.success() { success = true; break; }
            }
        }
    }
    if success && temp_db.exists() {
        let data = std::fs::read(&temp_db)?;
        let _ = std::fs::remove_file(&temp_db);
        return Ok(data);
    }
    let _ = std::fs::remove_file(&temp_db);
    Err("Failed to convert PS to PDF".into())
}
