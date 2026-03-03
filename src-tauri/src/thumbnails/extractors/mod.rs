pub mod ai;
pub mod aseprite;
pub mod binary_jpeg;
pub mod clip;
pub mod corel_painter;
pub mod coreldraw;
pub mod eps;
pub mod mdp;
pub mod penpot;
pub mod rebelle;
pub mod sai;
pub mod sai2;
pub mod sketch;
pub mod xcf;

use image::ImageEncoder;
use std::io::Read;
use std::path::Path;
use tauri::{AppHandle, Runtime};

/// Central registry for on-the-fly preview extraction.
pub fn extract_preview<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    path: &Path,
) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let format = crate::formats::FileFormat::detect(path).ok_or("Unsupported format")?;

    match format.preview_strategy {
        crate::formats::PreviewStrategy::BrowserNative => {
            Err("Browser native format - serve directly".into())
        }

        crate::formats::PreviewStrategy::Raw => {
            if let Ok((data, mime)) = extract_raw_preview(path) {
                return Ok((data, mime));
            }
            if let Ok(data) = extract_ffmpeg_frame(app_handle, path) {
                return Ok((data, "image/jpeg".to_string()));
            }
            Err("Failed all RAW preview extraction methods".into())
        }

        crate::formats::PreviewStrategy::Ffmpeg => {
            if let Ok(data) = extract_ffmpeg_frame(app_handle, path) {
                return Ok((data, "image/jpeg".to_string()));
            }
            let data = convert_to_png(path)?;
            Ok((data, "image/png".to_string()))
        }

        crate::formats::PreviewStrategy::NativeExtractor => {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            match ext.as_str() {
                // Affinity Suite
                "afphoto" | "afdesign" | "afpub" => {
                    let data = super::affinity::extract_largest_png(path)?;
                    Ok((data, "image/png".to_string()))
                }
                // Adobe Photoshop
                "psd" | "psb" => {
                    if let Ok(data) = extract_psd_composite(path) {
                        return Ok((data, "image/png".to_string()));
                    }
                    // Fallback to binary scanner
                    let (data, mime) = binary_jpeg::extract_any_embedded(path)?;
                    Ok((data, mime))
                }
                // Adobe Illustrator (PDF-based)
                "ai" => ai::extract_ai_preview(path),
                // Encapsulated PostScript / PostScript
                "eps" | "ps" => {
                    if let Ok((data, mime)) = eps::extract_eps_ps_preview(path) {
                        if mime == "image/tiff" {
                            if let Ok(png) = convert_to_png_from_memory(&data) {
                                return Ok((png, "image/png".to_string()));
                            }
                        }
                        return Ok((data, mime));
                    }

                    // Fallback to external process if native extraction fails.
                    // Priority: FFmpeg (relies on Ghostscript) for rendering vector
                    if let Ok(data) = extract_ffmpeg_frame(app_handle, path) {
                        return Ok((data, "image/jpeg".to_string()));
                    }

                    Err("No preview found in EPS/PS".into())
                }
                // ZIP-based Project Previews
                "xmind" => {
                    let data = extract_zip_preview(path)?;
                    Ok((data, "image/png".to_string()))
                }
                "clip" => clip::extract_clip_preview(path),
                "sketch" => sketch::extract_sketch_preview(path),
                "kra" | "krz" | "kra~" => {
                    let data = extract_krita_preview(path)?;
                    Ok((data, "image/png".to_string()))
                }
                "aseprite" | "ase" => aseprite::extract_aseprite_preview(path),
                "xcf" => xcf::extract_xcf_preview(path),
                "mdp" => mdp::extract_mdp_preview(path),
                // PaintTool SAI
                "sai" => sai::extract_sai_preview(path),
                // PaintTool SAI v2
                "sai2" => sai2::extract_sai2_preview(path),
                "reb" => rebelle::extract_rebelle_preview(path),
                "blend" => {
                    let (data, mime) = binary_jpeg::extract_any_embedded(path)?;
                    Ok((data, mime))
                }
                "hdr" | "exr" | "dds" => {
                    if let Ok(data) = convert_to_png(path) {
                        return Ok((data, "image/png".to_string()));
                    }
                    if let Ok(data) = extract_ffmpeg_frame(app_handle, path) {
                        return Ok((data, "image/jpeg".to_string()));
                    }
                    let (data, mime) = binary_jpeg::extract_any_embedded(path)?;
                    Ok((data, mime))
                }
                "fig" => {
                    let data = extract_figma_preview(path)?;
                    Ok((data, "image/png".to_string()))
                }
                "cdr" => coreldraw::extract_coreldraw_preview(path),
                "rif" | "riff" => corel_painter::extract_corel_painter_preview(path),
                "penpot" => {
                    let data = penpot::extract_penpot_preview(path)?;
                    Ok((data, "image/png".to_string()))
                }

                _ => Err("No native extractor for this extension".into()),
            }
        }

        crate::formats::PreviewStrategy::Convert => {
            let data = convert_to_png(path)?;
            Ok((data, "image/png".to_string()))
        }

        crate::formats::PreviewStrategy::None => Err("No preview strategy for this format".into()),
    }
}

/// Helper to extract a preview from a ZIP-based file.
fn extract_zip_preview(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let candidates = [
        "previews/preview.png",
        "Previews/preview.png",
        "Canvas/thumbnail.png",
        "Thumbnails/thumbnail.png",
        "Thumbnail/thumbnail.png",
        "QuickLook/Preview.png",
        "QuickLook/Thumbnail.png",
        "preview.png",
        "thumbnail.png",
        "icon.png",
    ];

    for name in candidates {
        if let Ok(mut entry) = archive.by_name(name) {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_lowercase();
        if entry_name.ends_with("preview.png") || entry_name.ends_with("thumbnail.png") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }

    Err("No preview found in zip archive".into())
}

/// Specialized extractor for Krita files.
fn extract_krita_preview(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // mergedimage.png is the full rendered canvas, best for preview
    if let Ok(mut entry) = archive.by_name("mergedimage.png") {
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        return Ok(buf);
    }

    // fallback to preview.png if mergedimage.png is missing (e.g. .krz or specific save settings)
    if let Ok(mut entry) = archive.by_name("preview.png") {
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        return Ok(buf);
    }

    Err("No valid preview (mergedimage.png or preview.png) found in Krita file".into())
}

/// Specialized extractor for Figma files (.fig).
/// Most modern .fig files (from 'Save local copy') are ZIP archives with a 'thumbnail.png' at the root.
fn extract_figma_preview(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;

    // Attempt to open as ZIP. Fallback to binary scanner if it's a legacy fig-kiwi file.
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(_) => {
            // Non-ZIP .fig files might have embedded JPEGs or thumbnails in XMP
            let (data, _) = binary_jpeg::extract_any_embedded(path)?;
            return Ok(data);
        }
    };

    if let Ok(mut entry) = archive.by_name("thumbnail.png") {
        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer)?;
        return Ok(buffer);
    }

    Err("No thumbnail.png found in Figma ZIP archive".into())
}

fn convert_to_png(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let sdr_img = img.to_rgb8();
    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    sdr_img.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(png_data)
}

fn extract_psd_composite(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let psd = psd::Psd::from_bytes(&bytes).map_err(|e| format!("PSD parse error: {}", e))?;

    let rgba = psd.rgba();
    let width = psd.width();
    let height = psd.height();

    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    image::codecs::png::PngEncoder::new(&mut cursor)
        .write_image(&rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG encode error: {}", e))?;

    Ok(png_data)
}

/// Helper to generate a thumbnail from extracted preview data.
pub fn generate_thumbnail_extracted<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    input_path: &Path,
    output_path: &Path,
    size_px: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut data, mime) = extract_preview(app_handle, input_path)?;

    // If it's a PDF (AI/EPS), we use PDFium for high-quality multiplatform rendering.
    if mime == "application/pdf" {
        // Try 1: Render Vector PDF (cleanest)
        if let Ok(rendered_data) =
            crate::media::pdf::render_pdf_data_to_image(app_handle, &data, size_px)
        {
            data = rendered_data;
        }
        // Try 2: Extract XMP Thumbnail (high priority especially for AI/EPS)
        else if let Ok(embedded_data) = ai::extract_xmp_thumbnail_safe(input_path) {
            data = embedded_data;
        }
        // Try 3: General Binary Scanner (fallback for non-XMP embedded images)
        else if let Ok((embedded_data, _)) = binary_jpeg::extract_any_embedded(input_path) {
            data = embedded_data;
        }
        // Try 4: FFmpeg Rasterization (slowest fallback)
        else if let Ok(rendered_data) = extract_ffmpeg_frame(app_handle, input_path) {
            data = rendered_data;
        } else {
            return Err("No PDF rendering or raster preview available for this file".into());
        }
    }

    process_extracted_image(&data, output_path, size_px)
}

fn process_extracted_image(
    data: &[u8],
    output_path: &Path,
    size_px: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    use fast_image_resize as fr;

    // Use zune-jpeg to bypass 'image' crate CMYK inversion bugs for Adobe XMP JPEGs.
    let (width, height, rgba_bytes) = if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        let mut decoder = zune_jpeg::JpegDecoder::new(data);
        if let Ok(pixels) = decoder.decode() {
            if let Some(info) = decoder.info() {
                let w = info.width as u32;
                let h = info.height as u32;
                let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);

                // zune-jpeg outputs RGB by default
                if pixels.len() == (w as usize) * (h as usize) * 3 {
                    for chunk in pixels.chunks_exact(3) {
                        rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                    }
                    (w, h, rgba)
                // Grayscale
                } else if pixels.len() == (w as usize) * (h as usize) {
                    for &gray in pixels.iter() {
                        rgba.extend_from_slice(&[gray, gray, gray, 255]);
                    }
                    (w, h, rgba)
                // Fallback (e.g. CMYK output explicitly configured, though defaults to RGB)
                } else if pixels.len() == (w as usize) * (h as usize) * 4 {
                    for chunk in pixels.chunks_exact(4) {
                        rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                    }
                    (w, h, rgba)
                } else {
                    let img = image::load_from_memory(data)?;
                    (img.width(), img.height(), img.to_rgba8().into_raw())
                }
            } else {
                let img = image::load_from_memory(data)?;
                (img.width(), img.height(), img.to_rgba8().into_raw())
            }
        } else {
            let img = image::load_from_memory(data)?;
            (img.width(), img.height(), img.to_rgba8().into_raw())
        }
    } else {
        let img = image::load_from_memory(data)?;
        (img.width(), img.height(), img.to_rgba8().into_raw())
    };

    let aspect = width as f32 / height as f32;
    let (new_w, new_h) = if aspect > 1.0 {
        (size_px, (size_px as f32 / aspect).max(1.0) as u32)
    } else {
        (((size_px as f32 * aspect).max(1.0)) as u32, size_px)
    };

    let src_image = fr::images::Image::from_vec_u8(width, height, rgba_bytes, fr::PixelType::U8x4)
        .map_err(|e| e.to_string())?;

    let mut dst_image = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);
    let mut resizer = fr::Resizer::new();
    resizer
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| e.to_string())?;

    let buffer = dst_image.buffer();
    crate::thumbnails::native::encode_webp_native(buffer, new_w, new_h, output_path)?;

    Ok(())
}

fn is_valid_image(data: &[u8]) -> bool {
    // Fast validation: try reading image dimensions from the payload.
    // This catches truncated JPEGs and unknown binary garbage masquerading as JPEGs.
    std::panic::catch_unwind(|| {
        let cursor = std::io::Cursor::new(data);
        if let Ok(reader) = image::ImageReader::new(cursor).with_guessed_format() {
            // into_dimensions reads just the header, it's very fast
            reader.into_dimensions().is_ok()
        } else {
            false
        }
    })
    .unwrap_or(false)
}

fn extract_raw_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    // 1. Try LibRaw first
    if let Ok(data) = crate::thumbnails::raw::extract_raw_preview_data(path) {
        let mime = if data.starts_with(&[0xFF, 0xD8]) {
            "image/jpeg"
        } else if data.starts_with(&[0x89, b'P', b'N', b'G']) {
            "image/png"
        } else if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
            || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        {
            "image/tiff"
        } else {
            "image/jpeg" // fallback guess
        };

        if mime == "image/tiff" {
            // Browsers cannot display TIFF natively. Convert to PNG to render successfully.
            if let Ok(png_data) = convert_to_png_from_memory(&data) {
                return Ok((png_data, "image/png".to_string()));
            }
        } else if is_valid_image(&data) {
            return Ok((data, mime.to_string()));
        }
    }

    // 2. Fallback to broad binary scanner (safely extracts JPEG/TIFF without truncation)
    if let Ok((data, mime)) = crate::thumbnails::extractors::binary_jpeg::extract_any_embedded(path)
    {
        if mime == "image/tiff" || mime == "image/x-tiff" {
            // Ensure browser compatibility
            if let Ok(png_data) = convert_to_png_from_memory(&data) {
                return Ok((png_data, "image/png".to_string()));
            }
        } else if is_valid_image(&data) {
            return Ok((data, mime));
        }
    }

    // 3. Fallback to brute-force JPEG scanner (safe for finding JPEGs inside complex binaries like .x3f or .raw)
    if let Ok(jpeg_data) = crate::thumbnails::raw::brute_force_extract_jpeg_data(path) {
        if is_valid_image(&jpeg_data) {
            return Ok((jpeg_data, "image/jpeg".to_string()));
        }
    }

    Err("Failed RAW preview extraction (No LibRaw thumb and no embedded binary found)".into())
}

fn extract_ffmpeg_frame<R: Runtime>(
    app_handle: Option<&AppHandle<R>>,
    path: &Path,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    crate::media::ffmpeg::extract_frame_to_memory(app_handle, path).map_err(|e| e.into())
}

fn convert_to_png_from_memory(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(data)?;
    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    img.to_rgb8()
        .write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(png_data)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_preview_issue() {
        let base = Path::new("../file-samples/Imagens/RAW");
        let ext_folders = vec!["3fr", "fff", "iiq", "kdc", "mef", "raw", "x3f"];

        for ext in ext_folders {
            let dir = base.join(ext);
            if dir.exists() {
                for entry in std::fs::read_dir(dir).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        tracing::debug!("\nTesting: {:?}", path);
                        match extract_raw_preview(&path) {
                            Ok((data, mime)) => {
                                tracing::debug!(
                                    "  extract_raw_preview OK: {} bytes, mime: {}",
                                    data.len(),
                                    mime
                                );
                                match image::load_from_memory(&data) {
                                    Ok(_) => {
                                        tracing::debug!("    -> image::load_from_memory SUCCESS")
                                    }
                                    Err(e) => {
                                        tracing::debug!(
                                            "    -> image::load_from_memory FAIL: {}",
                                            e
                                        )
                                    }
                                }
                            }
                            Err(e) => tracing::debug!("  extract_raw_preview ERR: {:?}", e),
                        }
                    }
                }
            }
        }
    }
}
