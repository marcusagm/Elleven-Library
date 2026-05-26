//! Corel Painter (.rif/.riff) preview extractor.
//!
//! Ported from V1 backend.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn extract_corel_painter_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 8];
    if file.read_exact(&mut header).is_err() {
        return Err("File too small".into());
    }
    if (header[0] != 0x00 || header[1] != 0x02) && &header[0..4] != b"RIFF" {
        return Err("Invalid header".into());
    }
    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buffer)?;
    let start_signature = [0xFF, 0xD8, 0xFF, 0xE0];
    if let Some(start_index) = buffer
        .windows(4)
        .position(|window| window == start_signature)
    {
        let end_signature = [0xFF, 0xD9];
        if let Some(end_relative_index) = buffer[start_index..]
            .windows(2)
            .position(|window| window == end_signature)
        {
            let end_index = start_index + end_relative_index + 2;
            return Ok((
                buffer[start_index..end_index].to_vec(),
                "image/jpeg".to_string(),
            ));
        }
    }
    Err("No embedded JPEG found".into())
}

pub fn extract_corelpainter_metadata(
    path: &Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut technical = serde_json::json!({
        "container": "Corel Painter RIF",
        "metadata_support": "Basic"
    });

    // Try to extract dimensions from the embedded JPEG preview as fallback
    if let Ok((preview_data, _mime_type)) = extract_corel_painter_preview(path) {
        if let Ok(reader) =
            image::ImageReader::new(std::io::Cursor::new(&preview_data)).with_guessed_format()
        {
            if let Ok((width, height)) = reader.into_dimensions() {
                technical["width"] = serde_json::json!(width);
                technical["height"] = serde_json::json!(height);
            }
        }
    }

    Ok(serde_json::json!({
        "technical": technical,
        "semantic": {}
    }))
}
