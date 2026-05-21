use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors;
use async_trait::async_trait;
use std::path::Path;

/// Provider for archive formats (.clip, .zip, .cbz).
#[derive(Default)]
pub struct ArchiveFormatProvider;

/// Implementation of `ArchiveFormatProvider`.
impl ArchiveFormatProvider {
    /// Create a new instance of `ArchiveFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Implementation of `FormatProvider` for `ArchiveFormatProvider`.
impl FormatProvider for ArchiveFormatProvider {
    /// Returns the name of the format provider.
    fn name(&self) -> &'static str {
        "ARCHIVE_FORMAT_PROVIDER"
    }

    /// Returns the supported extensions for the format provider.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - The supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["zip", "cbz", "clip", "rar", "7z", "tar", "gz"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy};

        vec![
            SupportedFormat::with_metadata(
                "ZIP Archive",
                vec!["zip"],
                vec!["application/zip"],
                MediaType::Archive,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Comic Book ZIP",
                vec!["cbz"],
                vec!["application/x-cbz"],
                MediaType::Archive,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Clip Studio Paint",
                vec!["clip"],
                vec!["application/x-clipstudio"],
                MediaType::Archive,
                ThumbnailStrategy::NativeExtractor,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "RAR Archive",
                vec!["rar"],
                vec!["application/vnd.rar"],
                MediaType::Archive,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "7-Zip Archive",
                vec!["7z"],
                vec!["application/x-7z-compressed"],
                MediaType::Archive,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "TAR Archive",
                vec!["tar"],
                vec!["application/x-tar"],
                MediaType::Archive,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "GZIP Archive",
                vec!["gz"],
                vec!["application/gzip"],
                MediaType::Archive,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Returns whether the format provider supports the given magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The magic bytes to check.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the format provider supports the given magic bytes.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // ZIP magic: PK
        header_bytes.starts_with(b"PK\x03\x04") || header_bytes.starts_with(b"CSFCHUNK")
    }

    /// Returns the thumbnail capability for the format provider.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn ThumbnailCapability>` - The thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementation of `ThumbnailCapability` for `ArchiveFormatProvider`.
#[async_trait]
impl ThumbnailCapability for ArchiveFormatProvider {
    /// Generate a thumbnail for the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `size_hint` - The size hint for the thumbnail.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - The thumbnail of the file.
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let extension = path_owned.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            if extension == "clip" {
                extractors::extract_clip_preview(&path_owned).map(|(d, _)| d)
                    .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))
            } else {
                extract_zip_thumbnail(&path_owned, size_hint)
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}


/// Helper: Extract thumbnail from regular ZIP/CBZ.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `size_hint` - The size hint for the thumbnail.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The thumbnail of the file.
fn extract_zip_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

    let preview_paths = [
        "preview.png",
        "Thumbnails/thumbnail.png",
        "QuickLook/Preview.png",
        "QuickLook/Thumbnail.png",
        "icon.png",
    ];

    for p in &preview_paths {
        if let Ok(mut entry) = archive.by_name(p) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(crate::core::error::AppError::Io)?;

            let img = image::load_from_memory(&buf)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;
            return crate::processing::media::extractors::image::process_and_encode_webp(img, size_hint);
        }
    }

    Err(crate::core::error::AppError::Generic(
        "No preview found in ZIP".into(),
    ))
}
