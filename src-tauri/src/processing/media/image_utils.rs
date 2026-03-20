//! Image utilities for validation and metadata extraction.
//!
//! Provides lightweight checks for image headers (PNG, JPEG, WebP)
//! without decoding the full image payload.

use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

/// Supported image formats for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl ImageFormat {
    /// Returns the MIME type for the format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Webp => "image/webp",
        }
    }
}

/// Checks if the provided bytes represent a valid supported image format.
///
/// Returns the detected format if valid, or None otherwise.
pub fn detect_image_format(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.len() < 12 {
        return None;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageFormat::Png);
    }

    // JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageFormat::Jpeg);
    }

    // WebP: RIFF ???? WEBP
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(ImageFormat::Webp);
    }

    None
}

/// Checks if the provided bytes are a valid image (PNG, JPEG, or WebP).
pub fn is_valid_image(bytes: &[u8]) -> bool {
    detect_image_format(bytes).is_some()
}

/// Attempts to extract image dimensions from the header without full decoding.
///
/// Supports PNG, JPEG (SOF markers), and WebP (VP8, VP8L, VP8X).
pub fn get_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let format = detect_image_format(bytes)?;
    let mut cursor = Cursor::new(bytes);

    match format {
        ImageFormat::Png => {
            // PNG dimensions are at offset 16 (4 bytes width, 4 bytes height, big-endian)
            if bytes.len() < 24 { return None; }
            let mut w_bytes = [0u8; 4];
            let mut h_bytes = [0u8; 4];
            w_bytes.copy_from_slice(&bytes[16..20]);
            h_bytes.copy_from_slice(&bytes[20..24]);
            Some((u32::from_be_bytes(w_bytes), u32::from_be_bytes(h_bytes)))
        }
        ImageFormat::Jpeg => {
            // Scan for SOF markers (0xFF 0xC0 through 0xFF 0xC3, 0xFF 0xC5 through 0xFF 0xCB, 0xFF 0xCD through 0xFF 0xCF)
            // Skip SOI (FF D8)
            let _ = cursor.seek(SeekFrom::Start(2));
            while let Ok(marker_start) = cursor.read_u8() {
                if marker_start != 0xFF { continue; }
                let marker = cursor.read_u8().ok()?;
                if marker == 0x00 || marker == 0xFF { continue; } // Padding or escaped FF
                
                let length = cursor.read_u16::<byteorder::BigEndian>().ok()?;
                if (0xC0..=0xC3).contains(&marker) || (0xC5..=0xCB).contains(&marker) || (0xCD..=0xCF).contains(&marker) {
                    let _precision = cursor.read_u8().ok()?;
                    let height = cursor.read_u16::<byteorder::BigEndian>().ok()? as u32;
                    let width = cursor.read_u16::<byteorder::BigEndian>().ok()? as u32;
                    return Some((width, height));
                } else {
                    let _ = cursor.seek(SeekFrom::Current((length as i64) - 2));
                }
            }
            None
        }
        ImageFormat::Webp => {
            if bytes.len() < 30 { return None; }
            let chunk_type = &bytes[12..16];
            match chunk_type {
                b"VP8 " => {
                    // VP8 (Lossy): 10 bytes offset, 3 bytes sync code, then width/height
                    let mut vp8_cursor = Cursor::new(&bytes[26..30]);
                    let w = vp8_cursor.read_u16::<LittleEndian>().ok()? & 0x3FFF;
                    let h = vp8_cursor.read_u16::<LittleEndian>().ok()? & 0x3FFF;
                    Some((w as u32, h as u32))
                }
                b"VP8L" => {
                    // VP8L (Lossless): 1 bit signature, 14 bits width, 14 bits height
                    let data = u32::from_le_bytes([bytes[21], bytes[22], bytes[23], bytes[24]]);
                    let w = (data & 0x3FFF) + 1;
                    let h = ((data >> 14) & 0x3FFF) + 1;
                    Some((w, h))
                }
                b"VP8X" => {
                    // VP8X (Extended): 24 bits width, 24 bits height
                    let w = (u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]) >> 0) + 1;
                    let h = (u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]) >> 0) + 1;
                    Some((w, h))
                }
                _ => None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        let png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        assert_eq!(detect_image_format(&png), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_webp() {
        let mut webp = b"RIFF\0\0\0\0WEBPVP8 ".to_vec();
        webp.extend_from_slice(&[0u8; 20]);
        assert_eq!(detect_image_format(&webp), Some(ImageFormat::Webp));
    }

    #[test]
    fn test_invalid_header() {
        let raw = vec![0u8; 12];
        assert_eq!(detect_image_format(&raw), None);
    }
}
