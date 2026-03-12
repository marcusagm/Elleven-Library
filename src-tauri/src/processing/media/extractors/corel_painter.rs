//! Corel Painter (.rif/.riff) preview extractor.
//!
//! Ported from V1 backend.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn extract_corel_painter_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 8];
    if file.read_exact(&mut header).is_err() { return Err("File too small".into()); }
    if header[0] != 0x00 || header[1] != 0x02 {
        if &header[0..4] != b"RIFF" { return Err("Invalid header".into()); }
    }
    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_end(&mut buffer)?;
    let start_sig = [0xFF, 0xD8, 0xFF, 0xE0];
    if let Some(start) = buffer.windows(4).position(|w| w == start_sig) {
        let end_sig = [0xFF, 0xD9];
        if let Some(end_rel) = buffer[start..].windows(2).position(|w| w == end_sig) {
            let end = start + end_rel + 2;
            return Ok((buffer[start..end].to_vec(), "image/jpeg".to_string()));
        }
    }
    Err("No embedded JPEG found".into())
}
