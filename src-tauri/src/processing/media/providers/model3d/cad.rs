use crate::core::error::AppResult;
use crate::core::formats::capabilities::MetadataCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for CAD engineering formats (.step, .stp, .iges, .igs).
///
/// Handles STEP (ISO 10303-21) and IGES interchange formats used in
/// mechanical engineering and industrial design. Currently provides
/// format recognition and basic metadata support.
///
/// # Technical Details
///
/// - **File Format**: STEP (ISO-10303-21 header), IGES (Section-based)
/// - **Thumbnail**: Not yet implemented
/// - **Preview**: Not yet implemented
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::model3d::cad::CadFormatProvider;
///
/// let provider = CadFormatProvider::new();
/// let extensions = provider.supported_extensions();
/// assert!(extensions.contains(&"step"));
/// assert!(extensions.contains(&"iges"));
/// ```
#[derive(Default)]
pub struct CadFormatProvider;

impl CadFormatProvider {
    /// Creates a new instance of `CadFormatProvider`.
    ///
    /// # Returns
    ///
    /// `CadFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for CadFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "CAD_FORMAT_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["step", "stp", "iges", "igs"]
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
                "STEP Model",
                vec!["step", "stp"],
                vec!["application/step"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "IGES Model",
                vec!["iges", "igs"],
                vec!["application/iges"],
                MediaType::Model3D,
                ThumbnailStrategy::None,
                PreviewStrategy::None,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Validates CAD magic bytes for STEP (`ISO-10303-21`) and IGES formats.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header matches a known CAD signature.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"ISO-10303-21")
            || header_bytes.starts_with(b"S      1")
            || header_bytes.starts_with(b"      ")
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for CadFormatProvider {
    /// Returns empty technical metadata (not yet implemented for CAD).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the CAD file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    #[instrument(skip(self, _path))]
    async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }

    /// Returns empty semantic metadata (not yet implemented for CAD).
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the CAD file (unused).
    ///
    /// # Errors
    ///
    /// This function always returns `Ok` with an empty JSON object.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}
