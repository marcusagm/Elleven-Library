use crate::core::formats::provider::{FormatProvider, SupportedFormat};

/// Provider for non-extractable archive formats (.rar, .7z, .tar, .gz).
///
/// These archive formats do not currently support thumbnail generation or
/// metadata extraction but are recognized by the system for indexing purposes.
///
/// # Technical Details
///
/// - **File Format**: RAR, 7-Zip, TAR, GZIP
/// - **Thumbnail**: Not supported (icon fallback)
/// - **Metadata**: Not supported
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::archive::compressed_archive::CompressedArchiveProvider;
///
/// let provider = CompressedArchiveProvider::new();
/// let extensions = provider.supported_extensions();
/// assert!(extensions.contains(&"rar"));
/// assert!(extensions.contains(&"7z"));
/// ```
#[derive(Default)]
pub struct CompressedArchiveProvider;

impl CompressedArchiveProvider {
    /// Creates a new instance of `CompressedArchiveProvider`.
    ///
    /// # Returns
    ///
    /// `CompressedArchiveProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for CompressedArchiveProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "COMPRESSED_ARCHIVE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["rar", "7z", "tar", "gz"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats with metadata.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![
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
}
