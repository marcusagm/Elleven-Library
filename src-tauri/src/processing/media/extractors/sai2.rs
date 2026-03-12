//! PaintTool SAI v2 (.sai2) preview extractor.
//!
//! Ported from V1 backend.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Sai2Error {
    #[error("Invalid SAI2 format: missing magic")]
    InvalidMagic,
    #[error("SAI2 header is too short")]
    HeaderTooShort,
    #[error("No thumbnail chunk found")]
    ThumbnailNotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

const SAI2_MAGIC: &[u8; 10] = b"SAI-CANVAS";
const HEADER_SIZE: usize = 64;
const CHUNK_DESCRIPTOR_SIZE: usize = 16;
const CANVAS_TYPE_THUMBNAIL_LOSSY: u32 = 0x11;
const CANVAS_TYPE_VIEW: u32 = 0x10;
const JSSF_MAGIC: &[u8; 4] = b"JSSF";
const JPEG_SOI_MARKER: [u8; 2] = [0xFF, 0xD8];

struct Sai2Header {
    chunk_count: u32,
}

fn parse_sai2_header<R: Read + Seek>(reader: &mut R) -> Result<Sai2Header, Sai2Error> {
    let mut header = [0u8; HEADER_SIZE];
    reader.seek(SeekFrom::Start(0))?;
    reader.read_exact(&mut header).map_err(|_| Sai2Error::HeaderTooShort)?;
    if &header[0..10] != SAI2_MAGIC { return Err(Sai2Error::InvalidMagic); }
    let chunk_count = u32::from_le_bytes([header[40], header[41], header[42], header[43]]);
    Ok(Sai2Header { chunk_count })
}

struct ChunkDescriptor {
    type_tag: [u8; 4],
    data_size: u64,
    data_offset: u64,
}

fn parse_chunk_list<R: Read + Seek>(reader: &mut R, chunk_count: u32) -> Result<Vec<ChunkDescriptor>, Sai2Error> {
    let mut running_offset = (HEADER_SIZE + (chunk_count as usize * CHUNK_DESCRIPTOR_SIZE)) as u64;
    reader.seek(SeekFrom::Start(HEADER_SIZE as u64))?;
    let mut descriptors = Vec::with_capacity(chunk_count as usize);
    for _ in 0..chunk_count {
        let mut buf = [0u8; CHUNK_DESCRIPTOR_SIZE];
        reader.read_exact(&mut buf)?;
        let mut type_tag = [0u8; 4];
        type_tag.copy_from_slice(&buf[0..4]);
        let data_size = u64::from_le_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]);
        descriptors.push(ChunkDescriptor { type_tag, data_size, data_offset: running_offset });
        running_offset += data_size;
    }
    Ok(descriptors)
}

struct CanvasDataEntry {
    canvas_type: u32,
    _data_size: u64,
    data_offset: u64,
}

fn iterate_canvas_data<R: Read + Seek>(reader: &mut R, descriptor: &ChunkDescriptor) -> Result<Vec<CanvasDataEntry>, Sai2Error> {
    let mut entries = Vec::new();
    let mut pos = descriptor.data_offset;
    let end = pos + descriptor.data_size;
    while pos < end {
        reader.seek(SeekFrom::Start(pos))?;
        let mut head = [0u8; 8];
        if reader.read_exact(&mut head).is_err() { break; }
        let canvas_type = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        let data_size = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as u64;
        if canvas_type == 0 { break; }
        entries.push(CanvasDataEntry { canvas_type, _data_size: data_size, data_offset: pos + 8 });
        pos = pos + 8 + data_size;
    }
    Ok(entries)
}

fn extract_jpeg<R: Read + Seek>(reader: &mut R, entry: &CanvasDataEntry) -> Result<Vec<u8>, Sai2Error> {
    reader.seek(SeekFrom::Start(entry.data_offset))?;
    let mut jssf = [0u8; 16];
    reader.read_exact(&mut jssf)?;
    if &jssf[0..4] != JSSF_MAGIC { return Err(Sai2Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSSF"))); }
    let size = u32::from_le_bytes([jssf[8], jssf[9], jssf[10], jssf[11]]) as usize;
    let mut data = vec![0u8; size];
    reader.read_exact(&mut data)?;
    if data.len() >= 2 && data[0..2] != JPEG_SOI_MARKER { return Err(Sai2Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JPEG"))); }
    Ok(data)
}

pub fn extract_sai2_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let header = parse_sai2_header(&mut file)?;
    let chunks = parse_chunk_list(&mut file, header.chunk_count)?;
    for chunk in chunks {
        if &chunk.type_tag == b"thum" || &chunk.type_tag == b"view" {
            let entries = iterate_canvas_data(&mut file, &chunk)?;
            for entry in entries {
                if entry.canvas_type == CANVAS_TYPE_THUMBNAIL_LOSSY || entry.canvas_type == CANVAS_TYPE_VIEW {
                    if let Ok(data) = extract_jpeg(&mut file, &entry) {
                        return Ok((data, "image/jpeg".to_string()));
                    }
                }
            }
        }
    }
    Err(Sai2Error::ThumbnailNotFound.into())
}
