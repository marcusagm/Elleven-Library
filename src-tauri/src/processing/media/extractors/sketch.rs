//! Sketch (.sketch) preview extractor.

use std::io::Read;
use std::path::Path;

pub fn extract_sketch_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for k in ["previews/preview.png", "Previews/preview.png"] {
        if let Ok(mut entry) = archive.by_name(k) {
            let mut buf = Vec::new(); entry.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        }
    }
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().to_lowercase().ends_with("preview.png") {
            let mut buf = Vec::new(); entry.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        }
    }
    Err("No preview in Sketch".into())
}
