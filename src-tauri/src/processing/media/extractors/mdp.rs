//! MediBang Paint / FireAlpaca (.mdp) preview extractor.
//!
//! Ported from V1 backend.

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use image::ImageEncoder;
use quick_xml::reader::Reader;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::Path;

pub fn extract_mdp_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 7];
    reader.read_exact(&mut magic)?;
    if &magic != b"mdipack" { return Err("Invalid MDP magic".into()); }
    reader.seek(SeekFrom::Current(5))?;
    let xml_len = reader.read_u32::<LittleEndian>()?;
    let _ = reader.read_u32::<LittleEndian>()?;
    let mut xml_buf = vec![0u8; xml_len as usize];
    reader.read_exact(&mut xml_buf)?;
    let xml = String::from_utf8_lossy(&xml_buf);

    let mut thumb_bin = String::new();
    let mut tw = 0; let mut th = 0;
    let mut xml_r = Reader::from_str(&xml);
    let mut buf = Vec::new();
    loop {
        match xml_r.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) | Ok(quick_xml::events::Event::Empty(e)) if e.name().as_ref() == b"Thumb" => {
                for a in e.attributes().flatten() {
                    match a.key.as_ref() {
                        b"bin" => thumb_bin = a.unescape_value()?.into_owned(),
                        b"width" => tw = a.unescape_value()?.parse()?,
                        b"height" => th = a.unescape_value()?.parse()?,
                        _ => {}
                    }
                }
                break;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    if thumb_bin.is_empty() { return Err("No thumbnail found in MDP".into()); }

    loop {
        let mut ph = [0u8; 132];
        if reader.read_exact(&mut ph).is_err() { break; }
        if &ph[0..4] != b"PAC " { break; }
        let t_size = u32::from_le_bytes([ph[4], ph[5], ph[6], ph[7]]);
        let t_flag = u32::from_le_bytes([ph[8], ph[9], ph[10], ph[11]]);
        let name = std::str::from_utf8(&ph[68..132])?.trim_matches(char::from(0));
        let d_len = t_size - 132;
        if name == thumb_bin {
            let mut raw = vec![0u8; d_len as usize];
            reader.read_exact(&mut raw)?;
            let pix = if t_flag == 1 {
                let mut dec = ZlibDecoder::new(&raw[..]);
                let mut out = Vec::new();
                dec.read_to_end(&mut out)?;
                out
            } else { raw };
            let mut rgba = pix;
            for c in rgba.chunks_exact_mut(4) { c.swap(0, 2); }
            let mut png = Vec::new();
            image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png)).write_image(&rgba, tw, th, image::ExtendedColorType::Rgba8)?;
            return Ok((png, "image/png".to_string()));
        } else { reader.seek(SeekFrom::Current(d_len as i64))?; }
    }
    Err("Thumbnail PAC block not found".into())
}
