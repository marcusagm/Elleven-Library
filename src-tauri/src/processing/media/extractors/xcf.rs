//! GIMP XCF (.xcf) preview extractor.
//!
//! Ported from V1 backend.

use byteorder::{BigEndian, ReadBytesExt};
use image::ImageEncoder;
use std::cmp;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XcfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid XCF format")]
    InvalidFormat,
    #[error("No layers found")]
    NoLayers,
}

struct LayerInfo {
    pointer: u64,
    width: u32,
    height: u32,
    offset_x: i32,
    offset_y: i32,
}

pub fn extract_xcf_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 9];
    reader.read_exact(&mut magic)?;
    if &magic != b"gimp xcf " { return Err(XcfError::InvalidFormat.into()); }

    let mut version_bytes = [0u8; 4];
    reader.read_exact(&mut version_bytes)?;
    let version = if &version_bytes == b"file" { 0 }
    else if version_bytes[0] == b'v' {
        std::str::from_utf8(&version_bytes[1..])?.parse::<u16>().unwrap_or(0)
    } else { 0 };

    reader.read_exact(&mut [0u8])?;
    let cw = reader.read_u32::<BigEndian>()?;
    let ch = reader.read_u32::<BigEndian>()?;
    let _ = reader.read_u32::<BigEndian>()?;
    if version >= 4 { reader.read_u32::<BigEndian>()?; }

    skip_properties(&mut reader)?;

    let bpo = if version >= 11 { 8 } else { 4 };
    let mut layer_ptrs = Vec::new();
    loop {
        let ptr = if bpo == 8 { reader.read_u64::<BigEndian>()? } else { reader.read_u32::<BigEndian>()? as u64 };
        if ptr == 0 { break; }
        layer_ptrs.push(ptr);
    }
    if layer_ptrs.is_empty() { return Err(XcfError::NoLayers.into()); }

    let mut visible_layers = Vec::new();
    for &ptr in &layer_ptrs {
        reader.seek(SeekFrom::Start(ptr))?;
        let w = reader.read_u32::<BigEndian>()?;
        let h = reader.read_u32::<BigEndian>()?;
        let _ = reader.read_u32::<BigEndian>()?;
        let _ = read_gimp_string(&mut reader)?;

        let mut vis = true;
        let mut ox = 0i32;
        let mut oy = 0i32;
        loop {
            let p_type = reader.read_u32::<BigEndian>()?;
            let p_len = reader.read_u32::<BigEndian>()?;
            if p_type == 0 { break; }
            match p_type {
                8 => vis = reader.read_u32::<BigEndian>()? != 0,
                15 => { ox = reader.read_i32::<BigEndian>()?; oy = reader.read_i32::<BigEndian>()?; }
                _ => { reader.seek(SeekFrom::Current(p_len as i64))?; }
            }
        }
        if vis {
            let hptr = if bpo == 8 { reader.read_u64::<BigEndian>()? } else { reader.read_u32::<BigEndian>()? as u64 };
            visible_layers.push(LayerInfo { pointer: hptr, width: w, height: h, offset_x: ox, offset_y: oy });
        }
    }

    let mut canvas = vec![0u8; (cw * ch * 4) as usize];
    visible_layers.reverse();
    for layer in visible_layers {
        if layer.pointer == 0 { continue; }
        reader.seek(SeekFrom::Start(layer.pointer))?;
        let _ = reader.read_u32::<BigEndian>()?;
        let _ = reader.read_u32::<BigEndian>()?;
        let bpp = reader.read_u32::<BigEndian>()?;
        if bpp != 3 && bpp != 4 { continue; }
        let lptr = if bpo == 8 { reader.read_u64::<BigEndian>()? } else { reader.read_u32::<BigEndian>()? as u64 };
        if lptr == 0 { continue; }
        reader.seek(SeekFrom::Start(lptr))?;
        let _ = reader.read_u32::<BigEndian>()?;
        let _ = reader.read_u32::<BigEndian>()?;

        let txs = layer.width.div_ceil(64);
        let tys = layer.height.div_ceil(64);
        for ty in 0..tys {
            for tx in 0..txs {
                reader.seek(SeekFrom::Start(lptr + 8 + ((ty * txs + tx) * bpo as u32) as u64))?;
                let tptr = if bpo == 8 { reader.read_u64::<BigEndian>()? } else { reader.read_u32::<BigEndian>()? as u64 };
                if tptr == 0 { continue; }
                let npos = reader.stream_position()?;
                reader.seek(SeekFrom::Start(tptr))?;
                decode_tile(&mut reader, &mut canvas, tx, ty, layer.width, layer.height, cw, ch, layer.offset_x, layer.offset_y, bpp)?;
                reader.seek(SeekFrom::Start(npos))?;
            }
        }
    }

    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png)).write_image(&canvas, cw, ch, image::ExtendedColorType::Rgba8)?;
    Ok((png, "image/png".to_string()))
}

fn skip_properties<R: Read + Seek>(reader: &mut R) -> Result<(), std::io::Error> {
    loop {
        let p_type = reader.read_u32::<BigEndian>()?;
        let p_len = reader.read_u32::<BigEndian>()?;
        if p_type == 0 { break; }
        reader.seek(SeekFrom::Current(p_len as i64))?;
    }
    Ok(())
}

fn read_gimp_string<R: Read>(reader: &mut R) -> Result<String, Box<dyn std::error::Error>> {
    let len = reader.read_u32::<BigEndian>()?;
    if len == 0 { return Ok(String::new()); }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    let end = buf.iter().position(|&b| b == 0).unwrap_or(len as usize);
    Ok(String::from_utf8_lossy(&buf[..end]).to_string())
}

#[allow(clippy::too_many_arguments)]
fn decode_tile<R: Read>(reader: &mut R, canvas: &mut [u8], tx: u32, ty: u32, lw: u32, lh: u32, cw: u32, ch: u32, ox: i32, oy: i32, bpp: u32) -> Result<(), Box<dyn std::error::Error>> {
    let xs = tx * 64;
    let ys = ty * 64;
    let tw = cmp::min(64, lw - xs);
    let th = cmp::min(64, lh - ys);
    let tp = tw * th;
    let mut trgba = vec![0u8; (tp * 4) as usize];
    if bpp == 3 { for i in 0..tp { trgba[(i * 4 + 3) as usize] = 255; } }
    for c in 0..bpp {
        let mut r = 0;
        while r < tp {
            let det = reader.read_u8()?;
            if det < 127 {
                let n = (det as u32) + 1;
                let v = reader.read_u8()?;
                for i in 0..n { if r + i < tp { trgba[((r + i) * 4 + c) as usize] = v; } }
                r += n;
            } else if det == 127 {
                let n = reader.read_u16::<BigEndian>()? as u32;
                let v = reader.read_u8()?;
                for i in 0..n { if r + i < tp { trgba[((r + i) * 4 + c) as usize] = v; } }
                r += n;
            } else if det == 128 {
                let n = reader.read_u16::<BigEndian>()? as u32;
                for i in 0..n { let v = reader.read_u8()?; if r + i < tp { trgba[((r + i) * 4 + c) as usize] = v; } }
                r += n;
            } else {
                let n = 256 - det as u32;
                for i in 0..n { let v = reader.read_u8()?; if r + i < tp { trgba[((r + i) * 4 + c) as usize] = v; } }
                r += n;
            }
        }
    }
    for t_y in 0..th {
        for t_x in 0..tw {
            let gx = ox + (xs + t_x) as i32;
            let gy = oy + (ys + t_y) as i32;
            if gx < 0 || gy < 0 || gx >= cw as i32 || gy >= ch as i32 { continue; }
            let ci = ((gy as u32 * cw + gx as u32) * 4) as usize;
            let ti = ((t_y * tw + t_x) * 4) as usize;
            let sa = trgba[ti + 3] as u32;
            if sa == 0 { continue; }
            let sr = trgba[ti] as u32; let sg = trgba[ti + 1] as u32; let sb = trgba[ti + 2] as u32;
            let dr = canvas[ci] as u32; let dg = canvas[ci + 1] as u32; let db = canvas[ci + 2] as u32; let da = canvas[ci + 3] as u32;
            let oa = sa + (da * (255 - sa) / 255);
            if oa > 0 {
                canvas[ci] = ((sr * sa + dr * da * (255 - sa) / 255) / oa) as u8;
                canvas[ci + 1] = ((sg * sa + dg * da * (255 - sa) / 255) / oa) as u8;
                canvas[ci + 2] = ((sb * sa + db * da * (255 - sa) / 255) / oa) as u8;
                canvas[ci + 3] = oa as u8;
            }
        }
    }
    Ok(())
}
