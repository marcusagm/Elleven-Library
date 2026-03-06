use super::capabilities::{MetadataCapability, ThumbnailCapability};

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
}
