
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use byteorder::{ReadBytesExt, LittleEndian};
use flate2::read::ZlibDecoder;

/// Entry point for generating a high-quality preview.
pub fn extract_coreldraw_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    println!("CDR [DEBUG]: Analyzing file: {:?}", path);
    let mut candidates: Vec<(Vec<u8>, String)> = Vec::new();

    // 1. Try format-specific extraction
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_ok() {
        // Modern ZIP
        if magic == [0x50, 0x4B, 0x03, 0x04] {
            println!("CDR [DEBUG]: Format detected: Modern ZIP");
             match extract_zip_best_quality(path) {
                Ok(res) => candidates.push(res),
                Err(e) => println!("CDR [DEBUG]: ZIP extraction failed: {}", e),
            }
        }
        // Legacy RIFF
        else if magic == *b"RIFF" {
            println!("CDR [DEBUG]: Format detected: Legacy RIFF");
             match extract_riff_previews(path) {
                Ok(mut res) => candidates.append(&mut res),
                Err(e) => println!("CDR [DEBUG]: RIFF extraction failed: {}", e),
            }
        }
        // WL.. signature (CorelDRAW v3-v5)
        else if magic[0] == 0x57 && magic[1] == 0x4C {
            println!("CDR [DEBUG]: Format detected: Legacy Corel (v3-v5) WL signature");
            if let Ok(thumb) = extract_wl_thumbnail(path) {
                 candidates.push(thumb);
            }
        }
    }

    // 2. Always run fallback "lucky scan" for embedded BMPs in binary files
    if candidates.is_empty() || is_legacy_format(path) {
         println!("CDR [DEBUG]: Running fallback BMP/Image scan...");
         match scan_for_embedded_images(path) {
             Ok(mut res) => candidates.append(&mut res),
             Err(_) => {}
         }
    }

    // 3. Select Best Candidate
    if let Some(best) = candidates.into_iter().max_by_key(|(data, _)| data.len()) {
        println!("CDR [DEBUG]: Best preview found. Size: {} bytes", best.0.len());
        if best.0.len() < 100 {
             return Err("Preview too small to be valid".into());
        }
        return Ok(best);
    }

    Err("No valid preview found in CorelDRAW file".into())
}

fn is_legacy_format(path: &Path) -> bool {
   if let Ok(mut file) = File::open(path) {
       let mut magic = [0u8; 4];
       if file.read_exact(&mut magic).is_ok() {
           return magic != [0x50, 0x4B, 0x03, 0x04];
       }
   }
   true
}

pub fn is_coreldraw(path: &Path) -> bool {
   if let Ok(mut file) = File::open(path) {
       let mut magic = [0u8; 4];
       if file.read_exact(&mut magic).is_ok() {
           return magic == [0x50, 0x4B, 0x03, 0x04] || magic == *b"RIFF";
       }
   }
   false
}

// --- MODERN (ZIP) STRATEGY ---

fn extract_zip_best_quality(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let candidates = [
        "previews/page1.png",
        "content/preview.png",
        "previews/thumbnail.png",
    ];

    let mut best_candidate: Option<(Vec<u8>, String)> = None;
    let mut max_size = 0;

    for candidate in candidates {
        if let Ok(mut entry) = archive.by_name(candidate) {
            let size = entry.size();
            println!("CDR [DEBUG]: Found candidate '{}' size: {} bytes", candidate, size);
            if size > max_size {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                max_size = size;
                best_candidate = Some((buf, "image/png".to_string()));
            }
        }
    }

    if let Some(candidate_data) = best_candidate {
        return Ok(candidate_data);
    }

    let mut best_entry_index = None;
    let mut max_size = 0;

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            if name.ends_with(".png") && (name.contains("preview") || name.contains("page") || name.contains("thumb")) {
                if entry.size() > max_size {
                    max_size = entry.size();
                    best_entry_index = Some(i);
                }
            }
        }
    }

    if let Some(i) = best_entry_index {
        let mut entry = archive.by_index(i)?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        return Ok((buf, "image/png".to_string()));
    }

    Err("No preview found in CorelDRAW ZIP container".into())
}


// --- LEGACY (RIFF) STRATEGY (RECURSIVE) ---

fn extract_riff_previews(path: &Path) -> Result<Vec<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(8))?;
    let mut sig = [0u8; 4];
    file.read_exact(&mut sig)?;

    if sig != *b"CDR " && sig != *b"CDRB" && sig != *b"CDRD" {
         return Err("Not a valid RIFF CDR file".into());
    }

    let file_len = file.metadata()?.len();
    let mut candidates = Vec::new();
    walk_riff_generic(&mut file, 12, file_len, &mut candidates)?;
    Ok(candidates)
}

fn walk_riff_generic<R: Read + Seek>(
    reader: &mut R,
    start_pos: u64,
    end_pos: u64,
    candidates: &mut Vec<(Vec<u8>, String)>
) -> Result<(), Box<dyn std::error::Error>> {
    reader.seek(SeekFrom::Start(start_pos))?;

    while reader.stream_position()? + 8 <= end_pos {
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() { break; }

        let chunk_size = reader.read_u32::<LittleEndian>()?;

        if chunk_size > 100_000_000 { break; }

        let next_chunk_pos = reader.stream_position()? + chunk_size as u64 + (chunk_size % 2) as u64;

        if chunk_id == *b"LIST" {
            let list_type_pos = reader.stream_position()?;
            let mut list_type = [0u8; 4];
            reader.read_exact(&mut list_type)?;

            if list_type == *b"disp" || list_type == *b"page" || list_type == *b"INFO" || list_type == *b"CMPR"
               || list_type == *b"doc " || list_type == *b"gobj" || list_type == *b"iccp" {
                 match walk_riff_generic(reader, list_type_pos + 4, next_chunk_pos, candidates) {
                     Ok(_) => {},
                     Err(_) => {
                         reader.seek(SeekFrom::Start(next_chunk_pos))?;
                     }
                 }
            } else if list_type == *b"cmpr" {
                 if chunk_size > 40 {
                     let mut compressed_data = vec![0u8; (chunk_size - 4) as usize];
                     reader.read_exact(&mut compressed_data)?;

                     if compressed_data.len() > 32 {
                         let zlib_offset = 24;
                         if zlib_offset < compressed_data.len() {
                             let zlib_stream = &compressed_data[zlib_offset..];
                             let mut decoder = ZlibDecoder::new(zlib_stream);
                             let mut decompressed = Vec::new();
                             if let Ok(_) = decoder.read_to_end(&mut decompressed) {
                                 let mut cursor = Cursor::new(decompressed);
                                 let len = cursor.get_ref().len() as u64;
                                 let _ = walk_riff_generic(&mut cursor, 0, len, candidates);
                             }
                         }
                     }
                 }
                 reader.seek(SeekFrom::Start(next_chunk_pos))?;
            } else {
                 reader.seek(SeekFrom::Start(next_chunk_pos))?;
            }

        } else if &chunk_id == b"DISP" || &chunk_id == b"icp0" || &chunk_id == b"bmp " || &chunk_id == b"imhd" {
             if chunk_size == 0 {
                 reader.seek(SeekFrom::Start(next_chunk_pos))?;
                 continue;
             }

             let mut data = vec![0u8; chunk_size as usize];
             reader.read_exact(&mut data)?;

             // Strategy: Try several BMP headers
             let mut found_bmp = false;

             // 1. Check for PNG/JPG/TIFF inside DISP
             if check_and_extract_image(&data, candidates) {
                 found_bmp = true;
             } else {
                 // 2. BMP variants
                 if data.len() > 2 && data[0] == b'B' && data[1] == b'M' {
                     candidates.push((data.clone(), "image/bmp".to_string()));
                     found_bmp = true;
                 }
                 else if data.len() > 6 && data[4] == b'B' && data[5] == b'M' {
                     candidates.push((data[4..].to_vec(), "image/bmp".to_string()));
                     found_bmp = true;
                 }
                 else if data.len() > 8 && data[4] == 0x28 && data[5] == 0x00 && data[6] == 0x00 && data[7] == 0x00 {
                     if let Ok(bmp) = construct_bmp_from_dib(&data[4..]) {
                         candidates.push((bmp, "image/bmp".to_string()));
                         found_bmp = true;
                     }
                 }
                 else if data.len() > 5 && data[1] == 0x28 && data[2] == 0x00 && data[3] == 0x00 {
                     if let Ok(bmp) = construct_bmp_from_dib(&data[1..]) {
                         candidates.push((bmp, "image/bmp".to_string()));
                         found_bmp = true;
                     }
                 }
                 else if data.len() > 4 && data[0] == 0x28 && data[1] == 0x00 && data[2] == 0x00 {
                     if let Ok(bmp) = construct_bmp_from_dib(&data) {
                         candidates.push((bmp, "image/bmp".to_string()));
                         found_bmp = true;
                     }
                 }
             }

             if !found_bmp {
                 // Could be unknown format or just raw pixels?
             }

             reader.seek(SeekFrom::Start(next_chunk_pos))?;
        } else {
            reader.seek(SeekFrom::Start(next_chunk_pos))?;
        }
    }
    Ok(())
}

fn check_and_extract_image(data: &[u8], candidates: &mut Vec<(Vec<u8>, String)>) -> bool {
    // Check for PNG
    if data.len() > 8 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        candidates.push((data.to_vec(), "image/png".to_string()));
        return true;
    }
    // Check for JPEG
    if data.len() > 2 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
         candidates.push((data.to_vec(), "image/jpeg".to_string()));
         return true;
    }
    // Check for TIFF (II*/MM*)
    if data.len() > 4 {
        if (data[0] == 0x49 && data[1] == 0x49 && data[2] == 0x2A && data[3] == 0x00) ||
           (data[0] == 0x4D && data[1] == 0x4D && data[2] == 0x00 && data[3] == 0x2A) {
             candidates.push((data.to_vec(), "image/tiff".to_string()));
             return true;
        }
    }
    // Check GIF
    if data.len() > 3 && data[0] == b'G' && data[1] == b'I' && data[2] == b'F' {
        candidates.push((data.to_vec(), "image/gif".to_string()));
        return true;
    }
    false
}

/// Constructs a full BMP file from a DIB (Device Independent Bitmap) memory block.
fn construct_bmp_from_dib(dib_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Write;
    if dib_data.len() < 4 { return Err("Too short".into()); }

    let header_size = u32::from_le_bytes([dib_data[0], dib_data[1], dib_data[2], dib_data[3]]);
    let header_size_usize = header_size as usize;
    if dib_data.len() < header_size_usize { return Err("DIB smaller than header".into()); }

    let dib_len = dib_data.len() as u32;
    let file_size = 14 + dib_len; // Approx

    // Parse BPP to guess palette size
    let mut palette_size = 0;

    // BPP is at offset 14 usually (if header >= 16 bytes)
    if header_size >= 16 && dib_data.len() >= 16 {
        let bpp = u16::from_le_bytes([dib_data[14], dib_data[15]]);
        if bpp <= 8 {
            let mut clr_used = 0;
            // clrUsed is at offset 32 if header >= 36
            if header_size >= 36 && dib_data.len() >= 36 {
                clr_used = u32::from_le_bytes([dib_data[32], dib_data[33], dib_data[34], dib_data[35]]);
            }

            if clr_used > 0 {
                palette_size = clr_used * 4;
            } else {
                palette_size = (1 << bpp) * 4;
            }
        }
    }

    let pixel_offset = 14 + header_size + palette_size;

    let mut bmp_data = Vec::with_capacity(file_size as usize);
    bmp_data.write_all(b"BM")?;
    bmp_data.write_all(&file_size.to_le_bytes())?;
    bmp_data.write_all(&[0u8; 4])?;
    bmp_data.write_all(&pixel_offset.to_le_bytes())?;
    bmp_data.write_all(dib_data)?;

    Ok(bmp_data)
}

/// Fallback: Scans the entire file for Embedded Images (BMP/PNG/JPG/TIFF).
fn scan_for_embedded_images(path: &Path) -> Result<Vec<(Vec<u8>, String)>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut found = Vec::new();
    let len = buffer.len();
    let mut i = 0;

    // Search for BMP and PNG only.
    // JPEG matching is disabled as 'FF D8' is too common in random binary data (false positives).

    while i < len.saturating_sub(16) {
        // BMP: 'BM' + Size check
        if buffer[i] == 0x42 && buffer[i+1] == 0x4D {
            let size_bytes = &buffer[i+2..i+6];
            let declared_size = u32::from_le_bytes(size_bytes.try_into()?) as usize;

            // Heuristic: Size must be reasonable for an embedded resource
            if declared_size > 50 && (i + declared_size) <= len {
                 let offset_val = u32::from_le_bytes(buffer[i+10..i+14].try_into()?) as usize;
                 // Header size check (offset usually 14 + header_size)
                 // Common offsets: 54 (v3), 14+header_size
                 if offset_val >= 14 && offset_val < 10000 {
                      let bmp_data = buffer[i..i+declared_size].to_vec();
                      found.push((bmp_data, "image/bmp".to_string()));
                      i += declared_size;
                      continue;
                 }
            }
        }

        // PNG: Strict Magic Check (8 bytes)
        // 89 50 4E 47 0D 0A 1A 0A
        if buffer[i] == 0x89 && buffer[i+1] == 0x50 && buffer[i+2] == 0x4E && buffer[i+3] == 0x47 &&
           buffer[i+4] == 0x0D && buffer[i+5] == 0x0A && buffer[i+6] == 0x1A && buffer[i+7] == 0x0A {

            // Scan for IEND to find end
            let mut end_pos = 0;
            // IEND chunk: Length(00 00 00 00) + 'IEND' + CRC(4 bytes) = 12 bytes total
            // But we just search for the 'IEND' tag and add 4 for CRC.
            for j in (i+8)..len.saturating_sub(8) {
                if buffer[j] == 0x49 && buffer[j+1] == 0x45 && buffer[j+2] == 0x4E && buffer[j+3] == 0x44 {
                    end_pos = j + 8; // IEND tag (4) + CRC (4) = end of chunk. (Tag starts at j)
                    break;
                }
            }
            if end_pos > 0 && end_pos <= len {
                let png_data = buffer[i..end_pos].to_vec();
                if png_data.len() > 60 { // Min PNG size
                     println!("CDR [DEBUG]: Found embedded PNG at {}. Size: {}", i, png_data.len());
                     found.push((png_data, "image/png".to_string()));
                     i = end_pos;
                     continue;
                }
            }
        }

        i += 1;
    }

    if found.is_empty() {
        return Err("No embedded images found".into());
    }
    Ok(found)
}

fn extract_wl_thumbnail(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut header = [0u8; 1024];
    file.read_exact(&mut header)?;

    let width = u16::from_be_bytes([header[0x48], header[0x49]]) as u32;
    let height = u16::from_be_bytes([header[0x4A], header[0x4B]]) as u32;

    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err("Invalid WL header dimensions".into());
    }

    println!("CDR [DEBUG]: WL Thumbnail detected: {}x{}", width, height);

    let stride = ((width + 31) / 32) * 4;
    let data_size = (stride * height) as usize;
    let start_offset = 0x56;

    if start_offset + data_size > header.len() {
        file.seek(SeekFrom::Start(0))?;
        let mut full_data = Vec::new();
        file.read_to_end(&mut full_data)?;
        if start_offset + data_size > full_data.len() { return Err("WL too short".into()); }
        let raw_bits = &full_data[start_offset..start_offset + data_size];
        return wrap_raw_1bit_bmp(raw_bits, width, height);
    }
    let raw_bits = &header[start_offset..start_offset + data_size];
    wrap_raw_1bit_bmp(raw_bits, width, height)
}

fn wrap_raw_1bit_bmp(raw_data: &[u8], width: u32, height: u32) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    use std::io::Write;
    let file_header_size = 14;
    let info_header_size = 40;
    let palette_size = 2 * 4;
    let offset = file_header_size + info_header_size + palette_size;
    let file_size = offset + raw_data.len() as u32;

    let mut bmp = Vec::with_capacity(file_size as usize);
    bmp.write_all(b"BM")?;
    bmp.write_all(&file_size.to_le_bytes())?;
    bmp.write_all(&[0u8; 4])?;
    bmp.write_all(&offset.to_le_bytes())?;
    bmp.write_all(&(info_header_size as u32).to_le_bytes())?;
    bmp.write_all(&(width as i32).to_le_bytes())?;
    bmp.write_all(&(-(height as i32)).to_le_bytes())?;
    bmp.write_all(&(1u16).to_le_bytes())?;
    bmp.write_all(&(1u16).to_le_bytes())?;
    bmp.write_all(&[0u8; 4])?;
    bmp.write_all(&(raw_data.len() as u32).to_le_bytes())?;
    bmp.write_all(&[0u8; 16])?;
    bmp.write_all(&[0x00, 0x00, 0x00, 0x00])?;
    bmp.write_all(&[0xFF, 0xFF, 0xFF, 0x00])?;
    bmp.write_all(raw_data)?;

    Ok((bmp, "image/bmp".to_string()))
}
