use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for MediBang Paint and FireAlpaca files (.mdp).
///
/// This provider handles the proprietary binary `mdipack` format used by MediBang Paint
/// and FireAlpaca. It extracts technical metadata (dimensions, resolution) and
/// semantic data (layer names) via a specialized internal extractor that parses
/// the embedded XML header and binary PAC blocks.
///
/// # Technical Details
///
/// ## File Format
///
/// The MDP file format is a proprietary binary format used by MediBang Paint and FireAlpaca.
/// It consists of a binary header followed by a series of data blocks. The header contains
/// information about the file, such as the file size, version, and canvas size. The data
/// blocks contain information about the layers, such as the layer name, layer type, and
/// layer data.
///
/// ## Magic Bytes
///
/// The magic bytes for the MDP file format are "mdipack".
///
/// ## File Structure
///
/// The MDP file format has the following structure:
///
/// ```text
/// +-----------------+
/// | Magic Bytes     |
/// +-----------------+
/// | Header          |
/// +-----------------+
/// | Data Blocks     |
/// +-----------------+
/// ```
///
/// ## Data Blocks
///
/// The data blocks are stored in the file in the following order:
///
/// ```text
/// +-----------------+
/// | Layer Data      |
/// +-----------------+
/// | Layer Data      |
/// +-----------------+
/// | Layer Data      |
/// +-----------------+
/// ```
///
/// ## File Extraction
///
/// The MDP file format is a proprietary format, so it requires a specialized
/// extractor to parse the file. The extractor parses the file and extracts the
/// technical metadata, such as the dimensions and resolution, and the semantic
/// data, such as the layer names and layer data.
///
/// # Examples
///
/// ```no_run
/// use mundam_lib::processing::media::providers::project::medibang::MedibangFormatProvider;
/// use mundam_lib::core::formats::provider::FormatProvider;
///
/// let provider = MedibangFormatProvider::new();
/// let supported_formats = provider.supported_formats();
///
/// assert!(!supported_formats.is_empty());
/// assert_eq!(provider.name(), "MEDIBANG_PROVIDER");
/// assert_eq!(provider.supported_extensions(), vec!["mdp"]);
/// ```
#[derive(Default)]
pub struct MedibangFormatProvider;

impl MedibangFormatProvider {
    /// Creates a new instance of `MedibangFormatProvider`.
    ///
    /// # Returns
    ///
    /// `MedibangFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for MedibangFormatProvider {
    /// Returns the unique name for this provider.
    fn name(&self) -> &'static str {
        "MEDIBANG_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["mdp"]
    }

    /// Returns the detailed format definitions supported by this provider.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "MediBang Paint / FireAlpaca",
            vec!["mdp"],
            vec!["application/x-medibang"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches the MediBang `mdipack` magic bytes.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"mdipack")
    }

    /// Returns the metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Returns the preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for MedibangFormatProvider {
    /// Extracts technical metadata such as dimensions and resolution.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MDP file.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let metadata = extractors::extract_mdp_metadata(&path_owned).map_err(
                |error: Box<dyn std::error::Error>| {
                    crate::core::error::AppError::Generic(error.to_string())
                },
            )?;

            Ok(metadata["technical"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata such as layer names.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MDP file.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let metadata = extractors::extract_mdp_metadata(&path_owned).map_err(
                |error: Box<dyn std::error::Error>| {
                    crate::core::error::AppError::Generic(error.to_string())
                },
            )?;

            Ok(metadata["semantic"].clone())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for MedibangFormatProvider {
    /// Generates a thumbnail for the MDP file by extracting the embedded preview block.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MDP file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Requested dimension for the thumbnail (currently unused for native previews).
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_mdp_preview(&path_owned)
                .map(|(image_data, _mime_type)| image_data)
                .map_err(|error: Box<dyn std::error::Error>| {
                    crate::core::error::AppError::Generic(error.to_string())
                })
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for MedibangFormatProvider {
    /// Generates a high-resolution PNG preview of the MDP file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MDP file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_mdp_preview(&path_owned).map_err(
                |error: Box<dyn std::error::Error>| {
                    crate::core::error::AppError::Generic(error.to_string())
                },
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
