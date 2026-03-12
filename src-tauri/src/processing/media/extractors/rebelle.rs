//! Rebelle (.reb) preview extractor.

use std::io::Read;
use std::path::Path;

pub fn extract_rebelle_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    if let Ok(mut entry) = archive.by_name("canvas.png") {
        let mut buf = Vec::new(); entry.read_to_end(&mut buf)?;
        return Ok((buf, "image/png".to_string()));
    }
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().eq_ignore_ascii_case("canvas.png") {
            let mut buf = Vec::new(); entry.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        }
    }
    Err("No canvas.png in Rebelle".into())
}
