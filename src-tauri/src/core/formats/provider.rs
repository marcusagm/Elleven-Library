use super::capabilities::{MetadataCapability, PreviewCapability, ThumbnailCapability};
use super::types::{MediaType, PlaybackStrategy, PreviewStrategy};
use serde::Serialize;

/// Represents a format supported by the application.
#[derive(Debug, Serialize, Clone)]
pub struct SupportedFormat {
    pub name: String,
    pub extensions: Vec<String>,
    pub mime_types: Vec<String>,
    pub type_category: MediaType,
    pub preview_strategy: PreviewStrategy,
    pub playback: PlaybackStrategy,
}

impl SupportedFormat {
    pub fn new(name: impl Into<String>, extensions: Vec<impl Into<String>>) -> Self {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(|e| e.into()).collect(),
            mime_types: Vec::new(),
            type_category: MediaType::Unknown,
            preview_strategy: PreviewStrategy::None,
            playback: PlaybackStrategy::None,
        }
    }

    pub fn with_metadata(
        name: &str,
        extensions: Vec<&str>,
        mime_types: Vec<&str>,
        type_category: MediaType,
        preview_strategy: PreviewStrategy,
        playback: PlaybackStrategy,
    ) -> Self {
        Self {
            name: name.to_string(),
            extensions: extensions.into_iter().map(|s| s.to_string()).collect(),
            mime_types: mime_types.into_iter().map(|s| s.to_string()).collect(),
            type_category,
            preview_strategy,
            playback,
        }
    }
}

/// A plugin that provides support for a specific file format.
///
/// Each `FormatProvider` declares which extensions it supports and
/// optionally provides capabilities like metadata extraction or thumbnail generation.
pub trait FormatProvider: Send + Sync {
    /// Returns the unique name of this provider.
    ///
    /// Examples: "ADOBE_PHOTOSHOP_PSD", "FFMPEG_VIDEO_FALLBACK".
    fn name(&self) -> &'static str;

    /// Returns the list of file extensions supported by this provider.
    ///
    /// Extension strings should not include the leading dot.
    /// These are used for O(1) routing in the `FormatRegistry`.
    fn supported_extensions(&self) -> Vec<&'static str>;

    /// Returns a list of granular formats supported by this provider.
    ///
    /// This allows a single provider to claim multiple logical formats
    /// (e.g. "JPEG Image", "PNG Image") instead of a single provider name.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        vec![SupportedFormat::new(
            self.name(),
            self.supported_extensions(),
        )]
    }

    /// Optional: Returns true if the provider can identify its format from magic bytes.
    ///
    /// # Arguments
    /// * `header_bytes` - The first bytes of the file.
    fn supports_magic_bytes(&self, _header_bytes: &[u8]) -> bool {
        false
    }

    /// Returns an optional reference to the metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        None
    }

    /// Returns an optional reference to the thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        None
    }

    /// Returns an optional reference to the high-res preview extraction capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        None
    }
}
