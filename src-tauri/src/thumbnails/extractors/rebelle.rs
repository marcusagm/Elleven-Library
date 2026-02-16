use std::path::Path;
use std::io::Read;
use std::fs::File;

/// Extracts the preview image from a Rebelle (.reb) file.
///
/// Rebelle files are ZIP archives containing a `canvas.png` file which represents
/// the full composite image of the artwork.
pub fn extract_rebelle_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // The 'canvas.png' file is the main composite image.
    // It is usually at the root of the archive.
    if let Ok(mut file) = archive.by_name("canvas.png") {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        return Ok((buffer, "image/png".to_string()));
    }

    // Fallback: check nested or case-insensitive if strictly needed,
    // but standard Rebelle files seem consistent.
    // Let's iterate just to be safe if strictly 'canvas.png' isn't found.
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().eq_ignore_ascii_case("canvas.png") {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            return Ok((buffer, "image/png".to_string()));
        }
    }
    Err("No canvas.png found in Rebelle file".into())
}
