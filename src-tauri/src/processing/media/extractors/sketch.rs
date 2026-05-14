//! Sketch (.sketch) preview and metadata extractor.

use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;

/// Extracts the preview image from a Sketch file.
///
/// Sketch files are ZIP archives containing a preview image at `previews/preview.png`.
///
/// # Arguments
///
/// * `path` - Path to the .sketch file.
///
/// # Returns
///
/// `Result<(Vec<u8>, String), Box<dyn std::error::Error>>` - The PNG image data and its MIME type.
pub fn extract_sketch_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let candidate_paths = ["previews/preview.png", "Previews/preview.png"];

    for candidate_path in candidate_paths {
        if let Ok(mut entry) = archive.by_name(candidate_path) {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }

    // Fallback: search for any file ending in preview.png
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.name().to_lowercase().ends_with("preview.png") {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }

    Err("No preview found in Sketch file".into())
}

/// Extracts technical and semantic metadata from a Sketch file.
///
/// Parses `meta.json` for versioning and page info, and uses the preview for dimensions.
///
/// # Arguments
///
/// * `path` - Path to the .sketch file.
///
/// # Returns
///
/// `Result<serde_json::Value, Box<dyn std::error::Error>>` - JSON containing technical and semantic data.
pub fn extract_sketch_metadata(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut technical_metadata = json!({
        "container": "ZIP",
    });
    let mut semantic_metadata = json!({});

    // Parse meta.json
    if let Ok(mut entry) = archive.by_name("meta.json") {
        let mut content = String::new();
        entry.read_to_string(&mut content)?;
        if let Ok(meta) = serde_json::from_str::<Value>(&content) {
            technical_metadata["app_version"] = meta["appVersion"].clone();
            technical_metadata["sketch_version"] = meta["version"].clone();
            technical_metadata["commit_hash"] = meta["commit"].clone();

            // Extract page names
            if let Some(pages) = meta["pagesAndArtboards"].as_object() {
                let page_names: Vec<_> = pages
                    .values()
                    .filter_map(|page| page["name"].as_str())
                    .map(|name| name.to_string())
                    .collect();
                semantic_metadata["pages"] = json!(page_names);
            }
        }
    }

    if let Ok((preview_data, _)) = extract_sketch_preview(path) {
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

