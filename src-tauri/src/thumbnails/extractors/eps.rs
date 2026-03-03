use crate::thumbnails::extractors::ai;
use crate::thumbnails::extractors::binary_jpeg;
use std::path::Path;

/// Main entry point for EPS and PS file preview extraction.
/// Implements native extraction strategies for thumbnails and previews.
/// Does not implement external process calling (Ghostscript via FFMPEG); that fallback
/// is handled centrally by the caller.
///
/// # Errors
/// Returns an error if no embedded imagery or PDF-wrapper can be found natively.
pub fn extract_eps_ps_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    // Priority 0: Official Binary EPS Header (Pointer-based TIFF)
    if let Ok((data, mime)) = binary_jpeg::extract_eps_binary_pointer(path) {
        return Ok((data, mime));
    }

    // Priority 1: High Quality Vector Conversion via OS tool (macOS: pstopdf, others: Ghostscript)
    // By turning the PS/EPS into a PDF, we feed it directly into the PDFium engine,
    // solving the "poor quality" preview issue for files that rely on XMP thumbnails.
    if let Ok(pdf_data) = convert_postscript_to_pdf_bytes(path) {
        return Ok((pdf_data, "application/pdf".to_string()));
    }

    // Priority 2: ASCII XMP Metadata Thumbnail (Fast but low-res JPEG, usually 256px)
    if let Ok(data) = ai::extract_xmp_thumbnail_safe(path) {
        return Ok((data, "image/png".to_string()));
    }

    // Priority 3: Fast Binary Scanner
    // (Finds raw embedded JPEGs or TIFF signatures without specific pointers)
    if let Ok((data, mime)) = binary_jpeg::extract_any_embedded(path) {
        return Ok((data, mime));
    }

    // Priority 4: Try to find a PDF stream directly embedded
    if let Ok(data) = ai::extract_ai_pdf_stream(path) {
        return Ok((data, "application/pdf".to_string()));
    }

    Err("No native embedded previews found in EPS/PS file".into())
}

/// Helper function to use OS shell tools (pstopdf or Ghostscript) to convert PostScript to PDF.
fn convert_postscript_to_pdf_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir();
    let unique_id = uuid::Uuid::new_v4().to_string();
    let temp_pdf = temp_dir.join(format!("mundam_eps_{}.pdf", unique_id));

    let mut success = false;

    // On macOS, Apple provides pstopdf out of the box
    if cfg!(target_os = "macos") {
        if let Ok(output) = std::process::Command::new("pstopdf")
            .arg(path)
            .arg("-o")
            .arg(&temp_pdf)
            .output()
        {
            if output.status.success() {
                success = true;
            }
        }
    }

    // Fallback to Ghostscript on all platforms (if pstopdf didn't succeed or wasn't run)
    if !success {
        #[cfg(target_os = "windows")]
        let gs_cmds = ["gswin64c", "gswin32c", "gs"];
        #[cfg(not(target_os = "windows"))]
        let gs_cmds = ["gs"];

        for cmd in gs_cmds.iter() {
            if let Ok(output) = std::process::Command::new(cmd)
                .arg("-sDEVICE=pdfwrite")
                .arg("-dSAFER")
                .arg("-dBATCH")
                .arg("-dNOPAUSE")
                .arg(format!("-sOutputFile={}", temp_pdf.display()))
                .arg(path)
                .output()
            {
                if output.status.success() {
                    success = true;
                    break;
                }
            }
        }
    }

    if success && temp_pdf.exists() {
        let pdf_data = std::fs::read(&temp_pdf)?;
        let _ = std::fs::remove_file(&temp_pdf);
        if !pdf_data.is_empty() {
            return Ok(pdf_data);
        }
    }

    let _ = std::fs::remove_file(&temp_pdf);
    Err("Failed to convert PostScript to PDF via OS tools".into())
}
