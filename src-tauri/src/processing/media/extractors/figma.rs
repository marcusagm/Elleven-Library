//! Figma (.fig) preview and metadata extractor.

use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;

/// Extracts the preview image from a Figma file.
///
/// Figma files are usually ZIP archives containing a `preview.png` or `thumbnail.png`.
///
/// # Arguments
///
/// * `path` - Path to the .fig file.
///
/// # Returns
///
/// `Result<(Vec<u8>, String), Box<dyn std::error::Error>>` - The PNG image data and its MIME type.
pub fn extract_figma_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let candidate_paths = ["preview.png", "thumbnail.png"];

    for candidate_path in candidate_paths {
        if let Ok(mut entry) = archive.by_name(candidate_path) {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }

    // Fallback: search for any PNG file that might be a preview
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.name().to_lowercase().ends_with(".png") {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }

    Err("No preview found in Figma file".into())
}

/// Extracts technical and semantic metadata from a Figma file.
///
/// Uses the preview for dimensions and looks for versioning info in the archive.
///
/// # Arguments
///
/// * `path` - Path to the .fig file.
///
/// # Returns
///
/// `Result<serde_json::Value, Box<dyn std::error::Error>>` - JSON containing technical and semantic data.
pub fn extract_figma_metadata(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut technical_metadata = json!({
        "container": "ZIP",
    });
    let semantic_metadata = json!({});

    // Check if it's a valid ZIP
    let file = std::fs::File::open(path)?;
    let archive = zip::ZipArchive::new(file)?;

    // Try to find a version or creator info in the ZIP comment or specific files
    if let Ok(comment) = String::from_utf8(archive.comment().to_vec()) {
        if !comment.is_empty() {
            technical_metadata["comment"] = json!(comment);
        }
    }

    // Use preview for dimensions
    if let Ok((preview_data, _)) = extract_figma_preview(path) {
        if let Ok(_reader) = image::ImageReader::new(std::io::Cursor::new(&preview_data)).with_guessed_format() {
            if let Ok((width, height)) = _reader.into_dimensions() {
                technical_metadata["width"] = width.into();
                technical_metadata["height"] = height.into();
            }
        }
    }

    Ok(json!({
        "technical": technical_metadata,
        "semantic": semantic_metadata,
    }))
}
