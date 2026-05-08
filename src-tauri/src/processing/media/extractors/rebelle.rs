//! Rebelle (.reb) preview extractor.

use std::io::Read;
use std::path::Path;

pub fn extract_rebelle_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Candidates priority based on Rebelle version history
    let preview_candidates = [
        ("canvas.png", "image/png"),
        ("thumbnail.jpg", "image/jpeg"),
        ("thumbnail.png", "image/png"),
        ("preview.png", "image/png"),
        ("preview.jpg", "image/jpeg"),
    ];

    // Phase 1: Try exact matches for known candidates
    for (candidate_name, mime_type) in &preview_candidates {
        if let Ok(mut entry) = archive.by_name(candidate_name) {
            let mut image_buffer = Vec::new();
            entry.read_to_end(&mut image_buffer)?;
            return Ok((image_buffer, mime_type.to_string()));
        }
    }

    // Phase 2: Case-insensitive search and fallback tracking
    let mut first_valid_image: Option<(Vec<u8>, String)> = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let entry_name = entry.name().to_lowercase();

        // Check against candidates case-insensitively
        for (candidate_name, mime_type) in &preview_candidates {
            if entry_name == candidate_name.to_lowercase() {
                let mut image_buffer = Vec::new();
                entry.read_to_end(&mut image_buffer)?;
                return Ok((image_buffer, mime_type.to_string()));
            }
        }

        // Fallback: capture first image found in the archive if nothing else matched
        if first_valid_image.is_none()
            && (entry_name.ends_with(".png")
                || entry_name.ends_with(".jpg")
                || entry_name.ends_with(".jpeg"))
        {
            let mut image_buffer = Vec::new();
            entry.read_to_end(&mut image_buffer)?;
            let detected_mime = if entry_name.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            first_valid_image = Some((image_buffer, detected_mime.to_string()));
        }
    }

    if let Some(fallback_image) = first_valid_image {
        return Ok(fallback_image);
    }

    Err("No valid preview or image found in Rebelle archive".into())
}

