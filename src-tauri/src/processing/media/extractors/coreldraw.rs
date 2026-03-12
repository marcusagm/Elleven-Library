//! CorelDRAW (.cdr) preview extractor.
//!
//! Ported and polished from V1. Supports Modern ZIP, Legacy RIFF, and WL thumbnails.

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub fn extract_coreldraw_preview(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut candidates = Vec::new();
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_ok() {
        if magic == [0x50, 0x4B, 0x03, 0x04] {
            if let Ok(res) = extract_zip_best(path) {
                candidates.push(res);
            }
        } else if magic == *b"RIFF" {
            if let Ok(mut res) = extract_riff_previews(path) {
                candidates.append(&mut res);
            }
        } else if magic[0] == 0x57 && magic[1] == 0x4C {
            if let Ok(res) = extract_wl_thumbnail(path) {
                candidates.push(res);
            }
        }
    }
    // Fallback image scan in mod.rs or reuse binary_jpeg?
    // For now, let's keep the specialized ones.

    if let Some(best) = candidates.into_iter().max_by_key(|(d, _)| d.len()) {
        if best.0.len() >= 100 {
            return Ok(best);
        }
    }
    Err("No valid preview found in CDR".into())
}

fn extract_zip_best(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let keys = [
        "previews/page1.png",
        "content/preview.png",
        "previews/thumbnail.png",
    ];
    for k in keys {
        if let Ok(mut e) = archive.by_name(k) {
            let mut buf = Vec::new();
            e.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        }
    }
    // Fallback: search for any PNG/large preview
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_lowercase();
        if name.ends_with(".png") && (name.contains("preview") || name.contains("page")) {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        }
    }
    Err("No ZIP preview".into())
}

fn extract_riff_previews(
    path: &Path,
) -> Result<Vec<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(8))?;
    let mut sig = [0u8; 4];
    file.read_exact(&mut sig)?;
    if sig != *b"CDR " && sig != *b"CDRB" && sig != *b"CDRD" {
        return Err("Not RIFF CDR".into());
    }
    let len = file.metadata()?.len();
    let mut res = Vec::new();
    walk_riff(&mut file, 12, len, &mut res)?;
    Ok(res)
}

fn walk_riff<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    res: &mut Vec<(Vec<u8>, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    reader.seek(SeekFrom::Start(start))?;
    while reader.stream_position()? + 8 <= end {
        let mut id = [0u8; 4];
        reader.read_exact(&mut id)?;
        let size = reader.read_u32::<LittleEndian>()?;
        if size > 100_000_000 {
            break;
        }
        let next = reader.stream_position()? + size as u64 + (size % 2) as u64;
        if id == *b"LIST" {
            let mut ltype = [0u8; 4];
            reader.read_exact(&mut ltype)?;
            if matches!(
                &ltype,
                b"disp" | b"page" | b"INFO" | b"CMPR" | b"doc " | b"gobj"
            ) {
                let current_pos = reader.stream_position()?;
                let _ = walk_riff(reader, current_pos, next, res);
            } else if ltype == *b"cmpr" && size > 40 {
                let mut comp = vec![0u8; (size - 4) as usize];
                reader.read_exact(&mut comp)?;
                if comp.len() > 32 {
                    let mut dec = ZlibDecoder::new(&comp[24..]);
                    let mut decomp = Vec::new();
                    if dec.read_to_end(&mut decomp).is_ok() {
                        let l = decomp.len() as u64;
                        let _ = walk_riff(&mut Cursor::new(decomp), 0, l, res);
                    }
                }
            }
        } else if matches!(&id, b"DISP" | b"icp0" | b"bmp " | b"imhd") && size > 0 {
            let mut data = vec![0u8; size as usize];
            reader.read_exact(&mut data)?;
            if data.starts_with(b"BM") || data.get(4..6) == Some(b"BM") {
                let start = if data.starts_with(b"BM") { 0 } else { 4 };
                res.push((data[start..].to_vec(), "image/bmp".to_string()));
            } else if data.len() > 8 && data[0] == 0x28 {
                // DIB
                if let Ok(bmp) = construct_bmp(&data) {
                    res.push((bmp, "image/bmp".to_string()));
                }
            }
        }
        reader.seek(SeekFrom::Start(next))?;
    }
    Ok(())
}

fn construct_bmp(dib: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if dib.len() < 4 {
        return Err("Too small".into());
    }
    let h_size = u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]);
    let mut pal = 0;
    if h_size >= 16 && dib.len() >= 16 {
        let bpp = u16::from_le_bytes([dib[14], dib[15]]);
        if bpp <= 8 {
            pal = (1 << bpp) * 4;
        }
    }
    let off = 14 + h_size + pal;
    let f_size = 14 + dib.len() as u32;
    let mut bmp = Vec::with_capacity(f_size as usize);
    bmp.write_all(b"BM")?;
    bmp.write_all(&f_size.to_le_bytes())?;
    bmp.write_all(&[0u8; 4])?;
    bmp.write_all(&off.to_le_bytes())?;
    bmp.write_all(dib)?;
    Ok(bmp)
}

fn extract_wl_thumbnail(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut head = [0u8; 1024];
    file.read_exact(&mut head)?;
    let w = u16::from_be_bytes([head[0x48], head[0x49]]) as u32;
    let h = u16::from_be_bytes([head[0x4A], head[0x4B]]) as u32;
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return Err("Invalid WL".into());
    }
    let stride = w.div_ceil(32) * 4;
    let d_size = (stride * h) as usize;
    let start = 0x56;
    if start + d_size > head.len() {
        file.seek(SeekFrom::Start(0))?;
        let mut full = Vec::new();
        file.read_to_end(&mut full)?;
        return wrap_wl(&full[start..start + d_size], w, h);
    }
    wrap_wl(&head[start..start + d_size], w, h)
}

fn wrap_wl(raw: &[u8], w: u32, h: u32) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let off = 14 + 40 + 8;
    let f_size = off + raw.len() as u32;
    let mut bmp = Vec::with_capacity(f_size as usize);
    bmp.write_all(b"BM")?;
    bmp.write_all(&f_size.to_le_bytes())?;
    bmp.write_all(&[0u8; 4])?;
    bmp.write_all(&off.to_le_bytes())?;
    bmp.write_all(&40u32.to_le_bytes())?;
    bmp.write_all(&(w as i32).to_le_bytes())?;
    bmp.write_all(&(-(h as i32)).to_le_bytes())?;
    bmp.write_all(&1u16.to_le_bytes())?; // Planes
    bmp.write_all(&1u16.to_le_bytes())?; // BPP
    bmp.write_all(&[0u8; 4])?; // Compression
    bmp.write_all(&(raw.len() as u32).to_le_bytes())?;
    bmp.write_all(&[0u8; 16])?;
    bmp.write_all(&[0x00, 0x00, 0x00, 0x00])?; // Palette 0
    bmp.write_all(&[0xFF, 0xFF, 0xFF, 0x00])?; // Palette 1
    bmp.write_all(raw)?;
    Ok((bmp, "image/bmp".to_string()))
}
