use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// PNG signature for magic byte detection.
const PNG_SIGNATURE: &[u8; 8] = b"\x89\x50\x4e\x47\x0d\x0a\x1a\x0a";
/// PNG IEND chunk for end of PNG data.
const PNG_IEND: &[u8; 4] = b"IEND";

/// Provider for Affinity files (.afphoto, .afdesign, .afpub)
pub struct AffinityFormatProvider;

/// Implementation of `AffinityFormatProvider`.
impl AffinityFormatProvider {
    /// Create a new instance of `AffinityFormatProvider`.
    ///
    /// # Returns
    ///
    /// A new instance of `AffinityFormatProvider`.
    pub fn new() -> Self {
        Self
    }

    /// Extract the largest PNG preview using binary signature scanning.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the PNG preview data.
    fn extract_largest_png(&self, path: &Path) -> AppResult<Vec<u8>> {
        let mut file = File::open(path).map_err(crate::core::error::AppError::Io)?;
        let file_size = file
            .metadata()
            .map_err(crate::core::error::AppError::Io)?
            .len();

        // Affinity files are usually large, previews are often at the end.
        let scan_size = 15 * 1024 * 1024; // 15MB scan window
        let start_offset = file_size.saturating_sub(scan_size);

        file.seek(SeekFrom::Start(start_offset))
            .map_err(crate::core::error::AppError::Io)?;

        let mut buffer = Vec::with_capacity((file_size - start_offset) as usize);
        file.read_to_end(&mut buffer)
            .map_err(crate::core::error::AppError::Io)?;

        let mut best_png: Option<(usize, usize)> = None;

        let mut i = 0;
        while i <= buffer.len().saturating_sub(8) {
            if &buffer[i..i + 8] == PNG_SIGNATURE {
                // Found PNG, limit search for IEND
                let search_limit = (i + 50 * 1024 * 1024).min(buffer.len());
                if let Some(iend_rel_offset) = self.find_iend(&buffer[i + 8..search_limit]) {
                    let png_length = iend_rel_offset + 8 + 4 + 4; // Signature + data until IEND + IEND + CRC

                    if best_png.is_none() || png_length > best_png.unwrap().1 {
                        best_png = Some((i, png_length));
                    }
                    i += png_length;
                    continue;
                }
            }
            i += 1;
        }

        if let Some((start, length)) = best_png {
            let end = (start + length).min(buffer.len());
            Ok(buffer[start..end].to_vec())
        } else {
            Err(crate::core::error::AppError::FormatNotSupported(
                "No PNG preview found in Affinity file".to_string(),
            ))
        }
    }

    /// Find the IEND chunk in the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to search for the IEND chunk.
    ///
    /// # Returns
    ///
    /// An `Option<usize>` containing the relative offset of the IEND chunk if found, `None` otherwise.
    fn find_iend(&self, data: &[u8]) -> Option<usize> {
        data.windows(4).position(|window| window == PNG_IEND)
    }
}

/// Trait for format provider.
impl FormatProvider for AffinityFormatProvider {
    /// Get the name of the format provider.
    ///
    /// # Returns
    ///
    /// A `&'static str` containing the name of the format provider.
    fn name(&self) -> &'static str {
        "AFFINITY_PROJECT_PROVIDER"
    }

    /// Get the supported extensions for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["afphoto", "afdesign", "afpub"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Affinity Photo Image",
                vec!["afphoto"],
                vec!["application/vnd.serif.affinity"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Affinity Designer Image",
                vec!["afdesign"],
                vec!["application/vnd.serif.affinity"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Affinity Publisher Image",
                vec!["afpub"],
                vec!["application/vnd.serif.affinity"],
                MediaType::Project,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Get the thumbnail capability for the format.
    ///
    /// # Returns
    ///
    /// An `Option<&dyn ThumbnailCapability>` containing the thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    // Affinity metadata is complex and binary, skipping for now as per sprint scope
}

/// Trait for thumbnail capability.
#[async_trait]
impl ThumbnailCapability for AffinityFormatProvider {
    /// Generate a thumbnail for the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `size_hint` - The desired size of the thumbnail.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the thumbnail data.
    async fn generate(&self, path: &Path, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        let provider = Self::new();

        tokio::task::spawn_blocking(move || provider.extract_largest_png(&path_owned))
            .await
            .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
