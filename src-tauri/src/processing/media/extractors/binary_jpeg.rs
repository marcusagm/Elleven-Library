//! Generic binary scanner for embedded images (JPEG, PNG, TIFF) and XMP thumbnails.
//!
//! Ported and improved from V1 backend.

use base64::{engine::general_purpose, Engine as _};
use image::ImageEncoder;
use memmap2::Mmap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const JPEG_SOI: &[u8; 2] = b"\xff\xd8";
const JPEG_EOI: &[u8; 2] = b"\xff\xd9";
const PNG_HEADER: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const PNG_FOOTER: &[u8; 4] = b"IEND";
const TIFF_LE: &[u8; 4] = b"II\x2a\x00";
const TIFF_BE: &[u8; 4] = b"MM\x00\x2a";

/// Scans for any embedded image (JPEG, PNG or TIFF), returning the largest one.
pub fn extract_any_embedded(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let mut best: Option<(Vec<u8>, String)> = None;

    if let Ok(data) = scan_mmap_for_jpeg(&mmap) {
        best = Some((data, "image/jpeg".to_string()));
    }

    if let Ok(data) = scan_mmap_for_png(&mmap) {
        if best.as_ref().is_none_or(|(old, _)| data.len() > old.len()) {
            best = Some((data, "image/png".to_string()));
        }
    }

    if let Ok(data) = scan_mmap_for_tiff(&mmap) {
        if best.as_ref().is_none_or(|(old, _)| data.len() > old.len()) {
            best = Some((data, "image/tiff".to_string()));
        }
    }

    if let Ok(data) = extract_xmp_thumbnail(path) {
        if best.as_ref().is_none_or(|(old, _)| data.len() > old.len()) {
            best = Some((data, "image/png".to_string()));
        }
    }

    best.ok_or_else(|| "No embedded image found".into())
}

pub fn scan_mmap_for_jpeg(mmap: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut best: Option<(usize, usize)> = None;
    let limit = mmap.len().min(30 * 1024 * 1024);
    let mut i = 0;
    while i < limit.saturating_sub(2) {
        if let Some(pos) = mmap[i..limit].windows(2).position(|w| w == JPEG_SOI) {
            let start = i + pos;
            let j = start + 2;
            let eoi_limit = (j + 20 * 1024 * 1024).min(mmap.len());
            if let Some(eoi_pos) = mmap[j..eoi_limit].windows(2).position(|w| w == JPEG_EOI) {
                let end = j + eoi_pos + 2;
                let len = end - start;
                if best.as_ref().is_none_or(|(_, bl)| len > *bl) {
                    best = Some((start, len));
                }
                i = end;
                continue;
            }
            i = start + 2;
        } else {
            break;
        }
    }
    best.map(|(s, l)| mmap[s..s + l].to_vec())
        .ok_or_else(|| "No JPEG found".into())
}

pub fn scan_mmap_for_png(mmap: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut best: Option<(usize, usize)> = None;
    let limit = mmap.len().min(30 * 1024 * 1024);
    let mut i = 0;
    while i < limit.saturating_sub(8) {
        if let Some(pos) = mmap[i..limit].windows(8).position(|w| w == PNG_HEADER) {
            let start = i + pos;
            let j = start + 8;
            if let Some(end_pos) = mmap[j..].windows(4).position(|w| w == PNG_FOOTER) {
                let end = j + end_pos + 8; // IEND + CRC
                let len = end - start;
                if best.as_ref().is_none_or(|(_, bl)| len > *bl) {
                    best = Some((start, len));
                }
                i = end.min(mmap.len());
                continue;
            }
            i = start + 8;
        } else {
            break;
        }
    }
    best.map(|(s, l)| mmap[s..s + l.min(mmap.len() - s)].to_vec())
        .ok_or_else(|| "No PNG found".into())
}

pub fn scan_mmap_for_tiff(mmap: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let limit = mmap.len().min(10 * 1024 * 1024);
    for i in 0..limit.saturating_sub(4) {
        if mmap[i..].starts_with(TIFF_LE) || mmap[i..].starts_with(TIFF_BE) {
            let end = if i == 0 {
                mmap.len()
            } else {
                (i + 50 * 1024 * 1024).min(mmap.len())
            };
            return Ok(mmap[i..end].to_vec());
        }
    }
    Err("No TIFF found".into())
}

pub fn extract_eps_binary_pointer(
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut head = [0u8; 32];
    file.read_exact(&mut head)?;
    if head[0..4] == [0xC5, 0xD0, 0xD3, 0xC6] {
        let off = u32::from_le_bytes(head[20..24].try_into()?) as u64;
        let len = u32::from_le_bytes(head[24..28].try_into()?) as u64;
        if len > 0 {
            file.seek(SeekFrom::Start(off))?;
            let mut data = vec![0u8; len as usize];
            file.read_exact(&mut data)?;
            return Ok((data, "image/tiff".to_string()));
        }
    }
    Err("No binary EPS header found".into())
}

pub fn extract_xmp_thumbnail(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut buf = Vec::new();
    file.take(10 * 1024 * 1024).read_to_end(&mut buf)?;

    let start_m = b"<xmpGImg:image>";
    let end_m = b"</xmpGImg:image>";

    if let Some(s_pos) = buf.windows(start_m.len()).position(|w| w == start_m) {
        let content_s = s_pos + start_m.len();
        if let Some(e_rel) = buf[content_s..]
            .windows(end_m.len())
            .position(|w| w == end_m)
        {
            let content_e = content_s + e_rel;
            let raw_str = String::from_utf8_lossy(&buf[content_s..content_e]);
            let clean: String = raw_str
                .replace("&#xA;", "")
                .replace("&#xD;", "")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
                .collect();
            let jpeg = general_purpose::STANDARD.decode(clean)?;

            // Adobe channel swap fix
            let mut decoder = zune_jpeg::JpegDecoder::new(&*jpeg);
            if decoder.decode_headers().is_ok() {
                if let Some(info) = decoder.info() {
                    let (w, h) = (info.width as u32, info.height as u32);
                    if let Ok(pix) = decoder.decode() {
                        let mut png = Vec::new();
                        let et = if pix.len() == (w * h * 4) as usize {
                            image::ExtendedColorType::Rgba8
                        } else if pix.len() == (w * h) as usize {
                            image::ExtendedColorType::L8
                        } else {
                            image::ExtendedColorType::Rgb8
                        };
                        if image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png))
                            .write_image(&pix, w, h, et)
                            .is_ok()
                        {
                            return Ok(png);
                        }
                    }
                }
            }
            return Ok(jpeg);
        }
    }
    Err("No XMP thumbnail found".into())
}
