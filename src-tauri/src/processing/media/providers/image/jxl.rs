use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::core::AppResult;
use async_trait::async_trait;
use std::path::Path;
use tracing::instrument;

/// Provider for JPEG XL image files (.jxl).
///
/// JPEG XL is a next-generation image format (ISO/IEC 18181) offering superior
/// compression, HDR support, progressive decoding, and lossless JPEG recompression.
///
/// This provider uses **`jxl-oxide`** — a 100% pure Rust decoder — for all
/// extraction operations, completely eliminating the dependency on FFmpeg having
/// `libjxl` compiled in. FFmpeg is retained only as a last-resort fallback.
///
/// # Technical Details
///
/// - **File Format**: JPEG XL (ISO/IEC 18181)
/// - **Decoder**: `jxl-oxide` (pure Rust, memory-safe, multithreaded via Rayon)
/// - **Thumbnail**: Native decode → resize → WebP (with FFmpeg fallback)
/// - **Preview**: Native decode → resize to 2048px → WebP (with FFmpeg fallback)
/// - **Metadata**: Dimensions, bit depth, color encoding, animation flags via
///   `jxl-oxide` header parsing + EXIF via `rexif`
///
/// # Supported Encapsulation Modes
///
/// - Bare codestream (magic: `FF 0A`)
/// - ISOBMFF container (magic: `00 00 00 0C 4A 58 4C 20`)
///
/// # Examples
///
/// ```rust
/// use mundam_core::formats::provider::FormatProvider;
/// use mundam_core::formats::types::MediaType;
///
/// let provider = JxlFormatProvider::new();
/// let formats = provider.supported_formats();
/// assert!(formats.contains(&MediaType::Image));
/// ```
#[derive(Default)]
pub struct JxlFormatProvider;

impl JxlFormatProvider {
    /// Creates a new instance of the JXL format provider.
    ///
    /// # Returns
    ///
    /// `JxlFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }
}

impl FormatProvider for JxlFormatProvider {
    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "JXL_IMAGE_PROVIDER"
    }

    /// Returns the list of file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["jxl"]
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

        vec![SupportedFormat::with_metadata(
            "JPEG XL Image",
            vec!["jxl"],
            vec!["image/jxl"],
            MediaType::Image,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates JXL magic bytes (codestream `FF 0A` or ISOBMFF `00 00 00 0C 4A 58 4C 20`).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - A byte slice containing the header bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the header bytes match JXL magic bytes, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"\xFF\x0A")
            || header_bytes.starts_with(b"\x00\x00\x00\x0C\x4A\x58\x4C\x20")
    }

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
    /// Returns the metadata extraction capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability for this provider.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for JxlFormatProvider {
    /// Extracts technical metadata from a JXL file using native `jxl-oxide` parsing.
    ///
    /// Reads the JXL header to extract dimensions, bit depth, color encoding,
    /// and animation flags. Also attempts EXIF extraction via `rexif` for
    /// containerized JXL files.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the JXL file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object containing the extracted metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::jxl::extract_jxl_metadata(&path_owned)
                .unwrap_or_else(|_| serde_json::json!({}))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)
    }

    /// Extracts semantic metadata from a JXL file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the JXL file.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - A JSON object containing the extracted metadata.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - Always returns an empty JSON object for now.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

#[async_trait]
impl ThumbnailCapability for JxlFormatProvider {
    /// Generates a WebP thumbnail from a JXL file using native `jxl-oxide` decoding.
    ///
    /// Falls back to FFmpeg if the native decoder fails.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the JXL file.
    /// * `asset_id` - The ID of the asset.
    /// * `size_hint` - The size hint for the thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - A vector of bytes containing the generated thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If all decoding tiers fail.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::jxl::generate_jxl_thumbnail(
                &path_owned,
                size_hint,
            )
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for JxlFormatProvider {
    /// Generates a high-quality preview from a JXL file using native `jxl-oxide` decoding.
    ///
    /// Falls back to FFmpeg if the native decoder fails.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JXL file.
    /// * `_asset_id` - Asset ID (not used in this implementation).
    ///
    /// # Returns
    ///
    /// `AppResult<(Vec<u8>, String)>` - A tuple containing the preview data and content type.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If all decoding tiers fail.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            crate::processing::media::extractors::jxl::extract_jxl_preview(&path_owned)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
