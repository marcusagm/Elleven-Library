//! Penpot project (.penpot) preview extractor.
//!
//! Ported from V1 backend.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn extract_penpot_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    if file.read(&mut magic)? < 4 { return Err("File too small".into()); }

    if magic == [0x50, 0x4B, 0x03, 0x04] {
        return extract_v1_zip(&mut file);
    }
    if magic == [0x01, 0x0B, 0x1A, 0x86] {
        return extract_v2_zstd(path);
    }
    Err("Unknown Penpot format".into())
}

fn extract_v1_zip(file: &mut File) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(0))?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut best_idx = None;
    let mut max_size = 0;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            if name.starts_with("objects/") && name.ends_with(".png") {
                if entry.size() > max_size {
                    max_size = entry.size();
                    best_idx = Some(i);
                }
            }
        }
    }
    if let Some(idx) = best_idx {
        let mut entry = archive.by_index(idx)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        return Ok((buf, "image/png".to_string()));
    }
    Err("No preview found in Penpot ZIP".into())
}

fn extract_v2_zstd(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(17))?;
    let decoder = zstd::stream::Decoder::new(file)?;
    let mut buf = Vec::new();
    decoder.take(50 * 1024 * 1024).read_to_end(&mut buf)?;
    if let Some(png) = scan_png(&buf) {
        return Ok((png, "image/png".to_string()));
    }
    Err("No preview found in Penpot Zstd stream".into())
}

fn scan_png(buf: &[u8]) -> Option<Vec<u8>> {
    let magic = b"\x89PNG\r\n\x1a\n";
    let mut best_png = None;
    let mut max_size = 0;
    let mut offset = 0;
    while offset < buf.len().saturating_sub(8) {
        if let Some(pos) = buf[offset..].windows(8).position(|w| w == magic) {
            let start = offset + pos;
            let mut cursor = start + 8;
            let mut iend = false;
            while cursor + 8 <= buf.len() {
                let len = u32::from_be_bytes([buf[cursor], buf[cursor+1], buf[cursor+2], buf[cursor+3]]) as usize;
                let tag = &buf[cursor+4..cursor+8];
                cursor += 12 + len;
                if tag == b"IEND" { iend = true; break; }
                if cursor > buf.len() { break; }
            }
            if iend && cursor - start > max_size {
                max_size = cursor - start;
                best_png = Some(buf[start..cursor].to_vec());
            }
            offset = start + 8;
        } else { break; }
    }
    best_png
}
