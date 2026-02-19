
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{ReadBytesExt, LittleEndian};

/// Entry point for generating a high-quality preview.
pub fn extract_coreldraw_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    println!("CDR [DEBUG]: Analyzing file: {:?}", path);
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    // Modern ZIP
    if magic == [0x50, 0x4B, 0x03, 0x04] {
        println!("CDR [DEBUG]: Format detected: Modern ZIP");
        return extract_zip_best_quality(path);
    }

    // Legacy RIFF
    if magic == *b"RIFF" {
        println!("CDR [DEBUG]: Format detected: Legacy RIFF");
        return extract_riff_preview_recursive(path);
    }

    println!("CDR [DEBUG]: Unknown magic: {:02X?}", magic);
    Err(format!("Unsupported CorelDRAW file signature: {:02X?}", magic).into())
}

/// Helper just for identifying if it's CorelDRAW (used by indexer)
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

    // Priority for Preview (High Quality):
    // 1. previews/page1.png (Full page render)
    // 2. content/preview.png
    // 3. previews/thumbnail.png

    let candidates = [
        "previews/page1.png",
        "content/preview.png",
        "previews/thumbnail.png",
    ];

    for candidate in candidates {
        if let Ok(mut entry) = archive.by_name(candidate) {
            println!("CDR [DEBUG]: Found candidate '{}' size: {} bytes", candidate, entry.size());
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok((buf, "image/png".to_string()));
        } else {
            println!("CDR [DEBUG]: Candidate '{}' not found", candidate);
        }
    }

    println!("CDR [DEBUG]: No standard candidates found. Scanning all files...");

    // Fallback: search for largest png
    let mut best_entry_index = None;
    let mut max_size = 0;

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();
            if name.ends_with(".png") && (name.contains("preview") || name.contains("page") || name.contains("thumb")) {
                println!("CDR [DEBUG]: Found potential preview '{}' size: {}", entry.name(), entry.size());
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

fn extract_riff_preview_recursive(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;

    // Check CDR signature at 0x8
    file.seek(SeekFrom::Start(8))?;
    let mut sig = [0u8; 4];
    file.read_exact(&mut sig)?;
    if sig != *b"CDR " && sig != *b"CDRB" {
         return Err("Not a valid RIFF CDR file".into());
    }

    let file_len = file.metadata()?.len();

    // Recursively walk from 12
    walk_riff(&mut file, 12, file_len)
}

fn walk_riff(file: &mut File, start_pos: u64, end_pos: u64) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(start_pos))?;

    while file.stream_position()? < end_pos - 8 {
        let mut chunk_id = [0u8; 4];
        if file.read_exact(&mut chunk_id).is_err() { break; }

        let chunk_size = file.read_u32::<LittleEndian>()?;
        let next_chunk_pos = file.stream_position()? + chunk_size as u64 + (chunk_size % 2) as u64;

        // println!("CDR [DEBUG]: Chunk '{}' size: {}", String::from_utf8_lossy(&chunk_id), chunk_size);

        if chunk_id == *b"LIST" {
            // Recurse into LIST
            // LIST structure: [LIST (4)] [Size (4)] [Type (4)] [SubChunks...]
            let list_type_pos = file.stream_position()?;
            let mut list_type = [0u8; 4];
            file.read_exact(&mut list_type)?;

            println!("CDR [DEBUG]: Entering LIST '{}'", String::from_utf8_lossy(&list_type));

            // Allow recursion if type matches known containers or just always recurse?
            // "INFO" lists contain metadata, maybe previews? "page" contains page data
            if list_type == *b"disp" || list_type == *b"page" || list_type == *b"INFO" || list_type == *b"CMPR" {
                 // Try to find DISP inside
                 match walk_riff(file, list_type_pos + 4, next_chunk_pos) {
                     Ok(res) => return Ok(res),
                     Err(_) => {
                         // Continue searching next chunks on this level
                         file.seek(SeekFrom::Start(next_chunk_pos))?;
                     }
                 }
            } else {
                 // Skip unknown list
                 file.seek(SeekFrom::Start(next_chunk_pos))?;
            }

        } else if &chunk_id == b"DISP" {
             println!("CDR [DEBUG]: Found DISP chunk. Size: {}", chunk_size);
             // Extract DISP data
             let mut data = vec![0u8; chunk_size as usize];
             file.read_exact(&mut data)?;

             // BMP with header offset (sometimes 4-byte generic header)
             // Header: [08, 00, 00, 00, 28, 00, 00, 00] -> The '28' is the size of BITMAPINFOHEADER (40 bytes)
             if data.len() > 8 && data[4] == 0x28 && data[5] == 0x00 && data[6] == 0x00 && data[7] == 0x00 {
                 println!("CDR [DEBUG]: Found BMP with 4-byte offset header (BITMAPINFOHEADER)");

                 // We need to construct a BMP File Header (14 bytes) + DIB Header + Pixel Data
                 // The data starting at offset 4 is the DIB Header (BITMAPINFOHEADER) followed by pixel data.
                 // We can construct a valid BMP file in memory.

                 let dib_header_slice = &data[4..];
                 let bmp = construct_bmp_from_dib(dib_header_slice);
                 if let Ok(bmp_data) = bmp {
                     return Ok((bmp_data, "image/bmp".to_string()));
                 }
                 println!("CDR [DEBUG]: Failed to construct BMP from DIB");
             }

             // Standard BMP check 'BM'
             if data.len() > 2 && data[0] == b'B' && data[1] == b'M' {
                 return Ok((data, "image/bmp".to_string()));
             }
             // BMP with 'BM' at offset 4 (rare but seen in code comments previously)
             if data.len() > 6 && data[4] == b'B' && data[5] == b'M' {
                 return Ok((data[4..].to_vec(), "image/bmp".to_string()));
             }

             // Check for WMF (Placeable Metafile Header: 0xD7CDC69A)
             // WMF is vector, hard to render. But we can identify it.
             if data.len() > 4 && data[0] == 0x9A && data[1] == 0xC6 && data[2] == 0xCD && data[3] == 0xD7 {
                  println!("CDR [DEBUG]: DISP is WMF (vector). Skipping.");
                  // Found WMF. This needs conversion.
                  // For now, return it as "image/wmf" and let the caller/frontend fail or use a converter if available.
                  // Our system doesn't have a WMF converter natively in Rust yet (unless we use image-rs with some plugin?)
                  // Actually, let's look for other chunks. Maybe there IS a BMP later?
                  // No, usually DISP is unique per view.
                  // return Err("Found DISP chunk but it is WMF (vector), not BMP (bitmap). Conversion not supported.".into());
             }

             // Continue if not identified
             println!("CDR [DEBUG]: DISP format not identified. Header: {:02X?}", &data[0..8.min(data.len())]);
             file.seek(SeekFrom::Start(next_chunk_pos))?;

        } else if &chunk_id == b"icp0" || &chunk_id == b"bmp " {
             println!("CDR [DEBUG]: Found Icon/BMP chunk '{}'", String::from_utf8_lossy(&chunk_id));
             let mut data = vec![0u8; chunk_size as usize];
             file.read_exact(&mut data)?;
             if data.len() > 2 && data[0] == b'B' && data[1] == b'M' {
                 return Ok((data, "image/bmp".to_string()));
             }
             file.seek(SeekFrom::Start(next_chunk_pos))?;
        } else {
            // println!("CDR [DEBUG]: Skipping chunk '{}'", String::from_utf8_lossy(&chunk_id));
            file.seek(SeekFrom::Start(next_chunk_pos))?;
        }
    }

    Err("No preview found in this RIFF level".into())
}

/// Constructs a full BMP file from a DIB (Device Independent Bitmap) memory block.
/// The DIB block usually starts with the BITMAPINFOHEADER.
fn construct_bmp_from_dib(dib_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Write;

    // BITMAPINFOHEADER is 40 bytes
    if dib_data.len() < 40 {
        return Err("DIB data too short".into());
    }

    // Parse width/height/bpp from header to calculate size if needed,
    // or just assume the rest of the data is the pixel array.
    // DIB Header layout (partial):
    // Offset 0 (u32): Header Size (40)
    // Offset 4 (i32): Width
    // Offset 8 (i32): Height
    // Offset 14 (u16): BPP

    // But for a simple file reconstruction, we just need to prepend the BMP File Header (14 bytes).
    // BMP File Header:
    // 0x00: "BM" (2 bytes)
    // 0x02: File Size (4 bytes)
    // 0x06: Reserved (2)
    // 0x08: Reserved (2)
    // 0x0A: Offset to Pixel Data (4 bytes)

    let dib_len = dib_data.len() as u32;
    let file_size = 14 + dib_len;
    // Offset to pixel data: 14 (FileHeader) + 40 (InfoHeader, usually) + Palette size (if any)
    // This is tricky without parsing.
    // However, many viewers (and `image` crate) might handle a pointer to DIB if we just wrap it with a basic header?

    // Let's try to be smart about the pixel offset.
    // Read BPP at offset 14 (u16)
    let bpp = u16::from_le_bytes([dib_data[14], dib_data[15]]);
    let mut palette_size = 0;

    if bpp <= 8 {
        // If clrUsed is crucial. Offset 32 (u32)
        let clr_used = u32::from_le_bytes([dib_data[32], dib_data[33], dib_data[34], dib_data[35]]);
        if clr_used > 0 {
            palette_size = clr_used * 4;
        } else {
            palette_size = (1 << bpp) * 4;
        }
    }

    let pixel_offset = 14 + 40 + palette_size; // assuming 40-byte header

    let mut bmp_data = Vec::with_capacity(file_size as usize);
    // Signature
    bmp_data.write_all(b"BM")?;
    // File Size
    bmp_data.write_all(&file_size.to_le_bytes())?;
    // Reserved
    bmp_data.write_all(&[0u8; 4])?;
    // Offset
    bmp_data.write_all(&pixel_offset.to_le_bytes())?;

    // Write the rest (DIB)
    bmp_data.write_all(dib_data)?;

    Ok(bmp_data)
}
