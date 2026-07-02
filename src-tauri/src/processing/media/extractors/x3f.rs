use crate::core::error::{AppError, AppResult};
use serde_json::{json, Map, Value};
use std::fs::File;
use std::path::Path;

/// Helper to parse Unicode UTF-16 LE null-terminated strings.
fn read_utf16_null_terminated_string(bytes: &[u8], start_offset: usize) -> Option<String> {
    let mut current_offset = start_offset;
    let mut utf16_units = Vec::new();

    while current_offset + 2 <= bytes.len() {
        let unit = u16::from_le_bytes([bytes[current_offset], bytes[current_offset + 1]]);
        if unit == 0 {
            break;
        }
        utf16_units.push(unit);
        current_offset += 2;
    }

    String::from_utf16(&utf16_units).ok()
}

/// Helper to parse X3F PROP section name-value pairs.
fn parse_x3f_properties_section(
    file_bytes: &[u8],
    data_offset: usize,
    _data_length: usize,
) -> Option<Map<String, Value>> {
    if data_offset + 24 > file_bytes.len() {
        return None;
    }
    if &file_bytes[data_offset..data_offset + 4] != b"SECp" {
        return None;
    }

    let number_of_properties = u32::from_le_bytes([
        file_bytes[data_offset + 8],
        file_bytes[data_offset + 9],
        file_bytes[data_offset + 10],
        file_bytes[data_offset + 11],
    ]) as usize;

    let character_format = u32::from_le_bytes([
        file_bytes[data_offset + 12],
        file_bytes[data_offset + 13],
        file_bytes[data_offset + 14],
        file_bytes[data_offset + 15],
    ]);

    // Character format 0 is UTF-16 LE
    if character_format != 0 {
        return None;
    }

    let entries_start_offset = data_offset + 24;
    let string_data_start_offset = entries_start_offset + number_of_properties * 8;

    if string_data_start_offset > file_bytes.len() {
        return None;
    }

    let mut properties_map = Map::new();

    for property_index in 0..number_of_properties {
        let entry_offset = entries_start_offset + property_index * 8;
        if entry_offset + 8 > file_bytes.len() {
            break;
        }

        let name_character_offset = u32::from_le_bytes([
            file_bytes[entry_offset],
            file_bytes[entry_offset + 1],
            file_bytes[entry_offset + 2],
            file_bytes[entry_offset + 3],
        ]) as usize;

        let value_character_offset = u32::from_le_bytes([
            file_bytes[entry_offset + 4],
            file_bytes[entry_offset + 5],
            file_bytes[entry_offset + 6],
            file_bytes[entry_offset + 7],
        ]) as usize;

        let name_byte_offset = string_data_start_offset + name_character_offset * 2;
        let value_byte_offset = string_data_start_offset + value_character_offset * 2;

        let name_string = read_utf16_null_terminated_string(file_bytes, name_byte_offset);
        let value_string = read_utf16_null_terminated_string(file_bytes, value_byte_offset);

        if let (Some(name), Some(value)) = (name_string, value_string) {
            let name_trimmed = name.trim().to_string();
            let value_trimmed = value.trim().to_string();
            if !name_trimmed.is_empty() && !value_trimmed.is_empty() {
                properties_map.insert(name_trimmed, Value::String(value_trimmed));
            }
        }
    }

    Some(properties_map)
}

/// Structure representing info parsed from X3F container directory entry.
struct X3FImageSectionInfo {
    payload_offset: usize,
    payload_length: usize,
    width: u32,
    height: u32,
}

/// Scans the X3F container directory for the embedded processed JPEG preview.
fn find_x3f_embedded_jpeg_preview_info(file_bytes: &[u8]) -> Option<X3FImageSectionInfo> {
    let total_file_length = file_bytes.len();
    if total_file_length < 4 || &file_bytes[0..4] != b"FOVb" {
        return None;
    }

    let directory_offset = u32::from_le_bytes([
        file_bytes[total_file_length - 4],
        file_bytes[total_file_length - 3],
        file_bytes[total_file_length - 2],
        file_bytes[total_file_length - 1],
    ]) as usize;

    if directory_offset + 12 > total_file_length {
        return None;
    }

    if &file_bytes[directory_offset..directory_offset + 4] != b"SECd" {
        return None;
    }

    let number_of_entries = u32::from_le_bytes([
        file_bytes[directory_offset + 8],
        file_bytes[directory_offset + 9],
        file_bytes[directory_offset + 10],
        file_bytes[directory_offset + 11],
    ]) as usize;

    let mut current_offset = directory_offset + 12;
    let mut best_section: Option<X3FImageSectionInfo> = None;
    let mut best_pixel_area = 0u64;

    for _entry_index in 0..number_of_entries {
        if current_offset + 12 > total_file_length {
            break;
        }

        let data_offset = u32::from_le_bytes([
            file_bytes[current_offset],
            file_bytes[current_offset + 1],
            file_bytes[current_offset + 2],
            file_bytes[current_offset + 3],
        ]) as usize;

        let data_length = u32::from_le_bytes([
            file_bytes[current_offset + 4],
            file_bytes[current_offset + 5],
            file_bytes[current_offset + 6],
            file_bytes[current_offset + 7],
        ]) as usize;

        let entry_type = &file_bytes[current_offset + 8..current_offset + 12];

        if (entry_type == b"IMA2" || entry_type == b"IMAG") && data_offset + 28 <= total_file_length
        {
            let section_identifier = &file_bytes[data_offset..data_offset + 4];
            if section_identifier == b"SECi" {
                let type_of_image_data = u32::from_le_bytes([
                    file_bytes[data_offset + 8],
                    file_bytes[data_offset + 9],
                    file_bytes[data_offset + 10],
                    file_bytes[data_offset + 11],
                ]);

                let data_format = u32::from_le_bytes([
                    file_bytes[data_offset + 12],
                    file_bytes[data_offset + 13],
                    file_bytes[data_offset + 14],
                    file_bytes[data_offset + 15],
                ]);

                let image_columns = u32::from_le_bytes([
                    file_bytes[data_offset + 16],
                    file_bytes[data_offset + 17],
                    file_bytes[data_offset + 18],
                    file_bytes[data_offset + 19],
                ]);

                let image_rows = u32::from_le_bytes([
                    file_bytes[data_offset + 20],
                    file_bytes[data_offset + 21],
                    file_bytes[data_offset + 22],
                    file_bytes[data_offset + 23],
                ]);

                // type_of_image_data 2 is processed preview, data_format 18 is JPEG
                if type_of_image_data == 2 && data_format == 18 {
                    let payload_start_offset = data_offset + 28;
                    let payload_end_offset = data_offset + data_length;

                    if payload_start_offset < payload_end_offset
                        && payload_end_offset <= total_file_length
                    {
                        let payload_length = payload_end_offset - payload_start_offset;
                        let pixel_area = (image_columns as u64) * (image_rows as u64);
                        if pixel_area > best_pixel_area {
                            best_pixel_area = pixel_area;
                            best_section = Some(X3FImageSectionInfo {
                                payload_offset: payload_start_offset,
                                payload_length,
                                width: image_columns,
                                height: image_rows,
                            });
                        }
                    }
                }
            }
        }

        current_offset += 12;
    }

    best_section
}

/// Extract custom properties and EXIF data from X3F container.
fn extract_x3f_properties_map(file_bytes: &[u8]) -> Map<String, Value> {
    let mut combined_properties_map = Map::new();

    // First, scan directory entries for "PROP" section
    let total_file_length = file_bytes.len();
    if total_file_length >= 4 && &file_bytes[0..4] == b"FOVb" {
        let directory_offset = u32::from_le_bytes([
            file_bytes[total_file_length - 4],
            file_bytes[total_file_length - 3],
            file_bytes[total_file_length - 2],
            file_bytes[total_file_length - 1],
        ]) as usize;

        if directory_offset + 12 <= total_file_length
            && &file_bytes[directory_offset..directory_offset + 4] == b"SECd"
        {
            let number_of_entries = u32::from_le_bytes([
                file_bytes[directory_offset + 8],
                file_bytes[directory_offset + 9],
                file_bytes[directory_offset + 10],
                file_bytes[directory_offset + 11],
            ]) as usize;

            let mut current_offset = directory_offset + 12;
            for _entry_index in 0..number_of_entries {
                if current_offset + 12 > total_file_length {
                    break;
                }

                let data_offset = u32::from_le_bytes([
                    file_bytes[current_offset],
                    file_bytes[current_offset + 1],
                    file_bytes[current_offset + 2],
                    file_bytes[current_offset + 3],
                ]) as usize;

                let data_length = u32::from_le_bytes([
                    file_bytes[current_offset + 4],
                    file_bytes[current_offset + 5],
                    file_bytes[current_offset + 6],
                    file_bytes[current_offset + 7],
                ]) as usize;

                let entry_type = &file_bytes[current_offset + 8..current_offset + 12];
                if entry_type == b"PROP" {
                    if let Some(parsed_properties) =
                        parse_x3f_properties_section(file_bytes, data_offset, data_length)
                    {
                        for (key, value) in parsed_properties {
                            combined_properties_map.insert(key, value);
                        }
                    }
                }

                current_offset += 12;
            }
        }
    }

    combined_properties_map
}

/// Extract technical metadata from the X3F file.
pub fn extract_x3f_metadata(path: &Path) -> AppResult<Value> {
    let file_handle = File::open(path).map_err(AppError::Io)?;
    let memory_map = unsafe {
        memmap2::MmapOptions::new()
            .map(&file_handle)
            .map_err(AppError::Io)?
    };

    let mut image_width = 0u32;
    let mut image_height = 0u32;
    let mut combined_metadata = Map::new();

    // 1. Get embedded JPEG info
    if let Some(image_section_info) = find_x3f_embedded_jpeg_preview_info(&memory_map) {
        image_width = image_section_info.width;
        image_height = image_section_info.height;

        let jpeg_payload = &memory_map[image_section_info.payload_offset
            ..image_section_info.payload_offset + image_section_info.payload_length];

        // Try to refine width and height using imagesize on the JPEG payload (highly accurate for preview)
        if let Ok(jpeg_dimensions) = imagesize::blob_size(jpeg_payload) {
            image_width = jpeg_dimensions.width as u32;
            image_height = jpeg_dimensions.height as u32;
        }

        // 2. Parse EXIF from the JPEG preview bytes
        if let Ok(exif_data) = rexif::parse_buffer(jpeg_payload) {
            for entry in exif_data.entries {
                let tag_name = entry.tag.to_string();
                let tag_value = entry.value_more_readable.to_string();
                let tag_value_trimmed = tag_value.trim().to_string();
                if !tag_value_trimmed.is_empty() {
                    combined_metadata.insert(tag_name, Value::String(tag_value_trimmed));
                }
            }
        }
    }

    // 3. Fallback/Supplement with custom X3F PROP section
    let custom_properties = extract_x3f_properties_map(&memory_map);
    for (key, value) in custom_properties {
        let mapped_key = match key.as_str() {
            "CAMMANUF" => "Make".to_string(),
            "CAMMODEL" => "Model".to_string(),
            "ISO" => "ISOSpeedRatings".to_string(),
            "APERTURE" => "FNumber".to_string(),
            "SH_DESC" => "ExposureTime".to_string(),
            "FLENGTH" => "FocalLength".to_string(),
            _ => key.clone(),
        };

        if !combined_metadata.contains_key(&mapped_key) {
            combined_metadata.insert(mapped_key, value);
        }
    }

    // Make sure we have Make/Model mapped properly
    if !combined_metadata.contains_key("Make") {
        if let Some(Value::String(camera_manufactuer)) = combined_metadata.get("CAMMANUF") {
            combined_metadata.insert(
                "Make".to_string(),
                Value::String(camera_manufactuer.clone()),
            );
        }
    }
    if !combined_metadata.contains_key("Model") {
        if let Some(Value::String(camera_model)) = combined_metadata.get("CAMMODEL") {
            combined_metadata.insert("Model".to_string(), Value::String(camera_model.clone()));
        }
    }

    // Calculate megapixels
    let megapixels = if image_width > 0 && image_height > 0 {
        Some(((image_width as f64 * image_height as f64) / 1_000_000.0 * 10.0).round() / 10.0)
    } else {
        None
    };

    let mut response_json = json!({
        "width": image_width,
        "height": image_height,
    });

    if let Some(megapixels_value) = megapixels {
        response_json["megapixels"] = json!(megapixels_value);
    }

    if let Some(response_object) = response_json.as_object_mut() {
        for (key, value) in combined_metadata {
            response_object.insert(key, value);
        }
    }

    Ok(response_json)
}

/// Extract the raw embedded JPEG preview bytes from X3F container.
pub fn extract_x3f_preview(path: &Path) -> AppResult<Vec<u8>> {
    let file_handle = File::open(path).map_err(AppError::Io)?;
    let memory_map = unsafe {
        memmap2::MmapOptions::new()
            .map(&file_handle)
            .map_err(AppError::Io)?
    };

    if let Some(image_section_info) = find_x3f_embedded_jpeg_preview_info(&memory_map) {
        let jpeg_payload = &memory_map[image_section_info.payload_offset
            ..image_section_info.payload_offset + image_section_info.payload_length];
        return Ok(jpeg_payload.to_vec());
    }

    Err(AppError::Generic(format!(
        "No processed JPEG preview found in X3F file {:?}",
        path
    )))
}

/// Generate a downscaled WebP thumbnail from the X3F container's embedded preview.
pub fn generate_x3f_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    let jpeg_bytes = extract_x3f_preview(path)?;
    let decoded_image = image::load_from_memory(&jpeg_bytes).map_err(|error| {
        AppError::Generic(format!(
            "Failed to decode embedded JPEG preview for thumbnail: {:?}",
            error
        ))
    })?;

    // Scale and encode as WebP
    crate::processing::media::extractors::image::process_and_encode_webp(decoded_image, size_hint)
}
