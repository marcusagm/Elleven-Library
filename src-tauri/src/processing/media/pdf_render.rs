use std::path::PathBuf;
use pdfium_render::prelude::*;

/// Renders a PDF dynamic byte slice into PNG image bytes using the bundled pdfium library.
///
/// # Arguments
///
/// * `pdf_data` - The PDF bytes slice to render.
/// * `size_hint` - Target maximum width or height.
///
/// # Returns
///
/// `Result<Vec<u8>, Box<dyn std::error::Error>>` - The PNG formatted image bytes on success.
pub fn render_pdf_to_png(pdf_data: &[u8], size_hint: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let library_path = find_pdfium_library_path()
        .ok_or_else(|| "Could not locate libpdfium binary")?;

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(library_path)
            .or_else(|binding_error| {
                tracing::warn!("Failed to bind to pdfium library, trying system library: {:?}", binding_error);
                Pdfium::bind_to_system_library()
            })?
    );

    let document = pdfium.load_pdf_from_byte_slice(pdf_data, None)?;

    if document.pages().is_empty() {
        return Err("PDF document contains no pages".into());
    }
    let page = document.pages().get(0)?;

    let size_hint_i32 = size_hint as i32;
    let render_config = PdfRenderConfig::new()
        .set_target_width(size_hint_i32)
        .set_maximum_height(size_hint_i32);

    let dynamic_image = page.render_with_config(&render_config)?.as_image();

    let mut byte_buffer = std::io::Cursor::new(Vec::new());
    dynamic_image.write_to(&mut byte_buffer, image::ImageFormat::Png)?;
    Ok(byte_buffer.into_inner())
}

/// Helper function to find libpdfium in macOS app bundle resources, executable path, or current working directory.
fn find_pdfium_library_path() -> Option<PathBuf> {
    let library_filename = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    if let Ok(executable_path) = std::env::current_exe() {
        let mut current_directory = executable_path.parent();
        while let Some(path) = current_directory {
            let macos_resources_path = path.join("Resources").join("binaries").join("pdfium").join(library_filename);
            if macos_resources_path.exists() {
                return Some(macos_resources_path);
            }

            let check_path_with_binaries = path.join("binaries").join("pdfium").join(library_filename);
            if check_path_with_binaries.exists() {
                return Some(check_path_with_binaries);
            }

            let check_path_without_binaries = path.join("pdfium").join(library_filename);
            if check_path_without_binaries.exists() {
                return Some(check_path_without_binaries);
            }

            current_directory = path.parent();
        }
    }

    if let Ok(current_working_directory) = std::env::current_dir() {
        let mut current_directory = Some(current_working_directory.as_path());
        while let Some(path) = current_directory {
            let check_path_with_binaries = path.join("binaries").join("pdfium").join(library_filename);
            if check_path_with_binaries.exists() {
                return Some(check_path_with_binaries);
            }
            let check_path_without_binaries = path.join("pdfium").join(library_filename);
            if check_path_without_binaries.exists() {
                return Some(check_path_without_binaries);
            }
            current_directory = path.parent();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const TEST_AI_PATH: &str = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Project/ai";

    #[test]
    fn test_render_pdf_from_ai() {
        let file_path = Path::new(TEST_AI_PATH).join("sample.ai");
        if file_path.exists() {
            let pdf_data = crate::processing::media::extractors::ai::extract_ai_pdf_stream(&file_path)
                .expect("Failed to extract PDF stream from AI file");
            assert!(!pdf_data.is_empty(), "PDF stream data should not be empty");

            let png_bytes = render_pdf_to_png(&pdf_data, 128)
                .expect("Failed to render PDF to PNG");
            assert!(!png_bytes.is_empty(), "Rendered PNG bytes should not be empty");

            // Verify it's a PNG header
            assert_eq!(&png_bytes[0..8], b"\x89PNG\r\n\x1a\n", "Rendered data must have PNG signature");
        }
    }
}
