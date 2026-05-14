//! MediBang Paint / FireAlpaca (.mdp) preview and metadata extractor.

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use image::ImageEncoder;
use quick_xml::reader::Reader;
use serde_json::json;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Information extracted from the MDP header XML.
#[derive(Debug, Default)]
pub struct MdpMetadata {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub create_time: String,
    pub update_time: String,
    pub layer_names: Vec<String>,
    pub thumbnail_bin: String,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

/// Parses the XML header of an MDP file to extract metadata.
///
/// # Arguments
///
/// * `reader` - A buffered reader for the MDP file.
///
/// # Returns
///
/// `Result<MdpMetadata, Box<dyn std::error::Error>>` - The extracted metadata.
fn parse_mdp_xml<R: Read + Seek>(
    reader: &mut BufReader<R>,
) -> Result<MdpMetadata, Box<dyn std::error::Error>> {
    let mut magic_buffer = [0u8; 7];
    reader.read_exact(&mut magic_buffer)?;
    if &magic_buffer != b"mdipack" {
        return Err("Invalid MDP magic: expected 'mdipack'".into());
    }

    // Skip 5 bytes (null padding in header)
    reader.seek(SeekFrom::Current(5))?;

    let xml_length = reader.read_u32::<LittleEndian>()?;
    let _unknown_field = reader.read_u32::<LittleEndian>()?;

    let mut xml_buffer = vec![0u8; xml_length as usize];
    reader.read_exact(&mut xml_buffer)?;
    let xml_content = String::from_utf8_lossy(&xml_buffer);

    let mut metadata = MdpMetadata::default();
    let mut xml_reader = Reader::from_str(&xml_content);
    let mut event_buffer = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut event_buffer) {
            Ok(quick_xml::events::Event::Start(element))
            | Ok(quick_xml::events::Event::Empty(element)) => {
                let name = element.name();
                match name.as_ref() {
                    b"Mdiapp" => {
                        for attribute in element.attributes().flatten() {
                            let key = attribute.key.as_ref();
                            let value = attribute.unescape_value()?;
                            match key {
                                b"width" => metadata.width = value.parse().unwrap_or(0),
                                b"height" => metadata.height = value.parse().unwrap_or(0),
                                b"dpi" => metadata.dpi = value.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    b"CreateTime" => {
                        for attribute in element.attributes().flatten() {
                            if attribute.key.as_ref() == b"timeString" {
                                metadata.create_time = attribute.unescape_value()?.into_owned();
                            }
                        }
                    }
                    b"UpdateTime" => {
                        for attribute in element.attributes().flatten() {
                            if attribute.key.as_ref() == b"timeString" {
                                metadata.update_time = attribute.unescape_value()?.into_owned();
                            }
                        }
                    }
                    b"Thumb" => {
                        for attribute in element.attributes().flatten() {
                            let key = attribute.key.as_ref();
                            let value = attribute.unescape_value()?;
                            match key {
                                b"bin" => metadata.thumbnail_bin = value.into_owned(),
                                b"width" => metadata.thumbnail_width = value.parse().unwrap_or(0),
                                b"height" => metadata.thumbnail_height = value.parse().unwrap_or(0),
                                _ => {}
                            }
                        }
                    }
                    b"Layer" => {
                        for attribute in element.attributes().flatten() {
                            if attribute.key.as_ref() == b"name" {
                                metadata
                                    .layer_names
                                    .push(attribute.unescape_value()?.into_owned());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        event_buffer.clear();
    }

    Ok(metadata)
}

/// Extracts technical and semantic metadata from an MDP file.
///
/// # Arguments
///
/// * `path` - Path to the MDP file.
///
/// # Returns
///
/// `Result<serde_json::Value, Box<dyn std::error::Error>>` - JSON containing technical and semantic data.
pub fn extract_mdp_metadata(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let metadata = parse_mdp_xml(&mut reader)?;

    Ok(json!({
        "technical": {
            "width": metadata.width,
            "height": metadata.height,
            "dpi": metadata.dpi,
            "create_time": metadata.create_time,
            "update_time": metadata.update_time,
        },
        "semantic": {
            "layer_names": metadata.layer_names,
        }
    }))
}

/// Extracts the preview image from an MDP file.
///
/// # Arguments
///
/// * `path` - Path to the MDP file.
///
/// # Returns
///
/// `Result<(Vec<u8>, String), Box<dyn std::error::Error>>` - The PNG image data and its MIME type.
pub fn extract_mdp_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    let metadata = parse_mdp_xml(&mut reader)?;
    if metadata.thumbnail_bin.is_empty() {
        return Err("No thumbnail bin identifier found in MDP XML".into());
    }

    // PAC blocks follow the XML header
    loop {
        let mut pac_header = [0u8; 132];
        if reader.read_exact(&mut pac_header).is_err() {
            break;
        }

        if &pac_header[0..4] != b"PAC " {
            break;
        }

        let total_block_size =
            u32::from_le_bytes([pac_header[4], pac_header[5], pac_header[6], pac_header[7]]);
        let compression_flag =
            u32::from_le_bytes([pac_header[8], pac_header[9], pac_header[10], pac_header[11]]);
        let block_name = std::str::from_utf8(&pac_header[68..132])?.trim_matches(char::from(0));

        let data_length = total_block_size - 132;

        if block_name == metadata.thumbnail_bin {
            let mut raw_data = vec![0u8; data_length as usize];
            reader.read_exact(&mut raw_data)?;

            let pixels = if compression_flag == 1 {
                let mut decoder = ZlibDecoder::new(&raw_data[..]);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                decompressed
            } else {
                raw_data
            };

            // MDP thumbnails are BGRA, convert to RGBA
            let mut rgba_pixels = pixels;
            for chunk in rgba_pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }

            let mut png_data = Vec::new();
            image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png_data)).write_image(
                &rgba_pixels,
                metadata.thumbnail_width,
                metadata.thumbnail_height,
                image::ExtendedColorType::Rgba8,
            )?;

            return Ok((png_data, "image/png".to_string()));
        } else {
            // Seek to the next PAC block
            reader.seek(SeekFrom::Current(data_length as i64))?;
        }
    }

    Err("Thumbnail PAC block not found in MDP file".into())
}
