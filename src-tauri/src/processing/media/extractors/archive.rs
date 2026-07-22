//! Extraction utilities for ZIP-based archive formats.
//!
//! Provides shared thumbnail extraction logic for archives that contain
//! embedded preview images (such as ZIP, CBZ, and container-based project files).

use crate::core::error::AppResult;

/// Extracts a thumbnail image from a ZIP archive by searching for common
/// preview file paths.
///
/// Looks for preview images in well-known paths inside the archive:
/// `preview.png`, `Thumbnails/thumbnail.png`, `QuickLook/Preview.png`,
/// `QuickLook/Thumbnail.png`, and `icon.png`.
///
/// # Arguments
///
/// * `path` - The path to the ZIP archive on disk.
/// * `size_hint` - The desired maximum dimension for the thumbnail.
///
/// # Returns
///
/// The thumbnail bytes encoded as WebP.
///
/// # Errors
///
/// * `AppError::Io` - If the file cannot be opened.
/// * `AppError::Generic` - If no preview image is found inside the archive,
///   or if decoding/encoding fails.
pub fn extract_zip_thumbnail(path: &std::path::Path, size_hint: u32) -> AppResult<Vec<u8>> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

    let preview_paths = [
        "preview.png",
        "Thumbnails/thumbnail.png",
        "QuickLook/Preview.png",
        "QuickLook/Thumbnail.png",
        "icon.png",
    ];

    for preview_path in &preview_paths {
        if let Ok(mut entry) = archive.by_name(preview_path) {
            let mut image_buffer = Vec::new();
            entry
                .read_to_end(&mut image_buffer)
                .map_err(crate::core::error::AppError::Io)?;

            let decoded_image = image::load_from_memory(&image_buffer)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;
            return crate::processing::media::extractors::image::process_and_encode_webp(
                decoded_image,
                size_hint,
            );
        }
    }

    Err(crate::core::error::AppError::Generic(
        "No preview found in ZIP".into(),
    ))
}
