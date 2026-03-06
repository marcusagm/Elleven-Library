use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use resvg::usvg;
use std::path::Path;
use tiny_skia::Pixmap;

/// Provider for SVG vector files
pub struct SvgFormatProvider;

/// Implementation of `SvgFormatProvider`.
impl SvgFormatProvider {
    /// Create a new instance of `SvgFormatProvider`.
    ///
    /// # Returns
    ///
    /// A new instance of `SvgFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Trait for format provider.
impl FormatProvider for SvgFormatProvider {
    /// Get the name of the format provider.
    ///
    /// # Returns
    ///
    /// A `&'static str` containing the name of the format provider.
    fn name(&self) -> &'static str {
        "SVG_VECTOR_PROVIDER"
    }

    /// Get the supported extensions for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["svg"]
    }

    /// Check if the given header bytes support the format.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The header bytes of the file.
    ///
    /// # Returns
    ///
    /// `true` if the format is supported, `false` otherwise.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"<?xml") || header_bytes.starts_with(b"<svg")
    }

    /// Get the thumbnail capability for the format.
    ///
    /// # Returns
    ///
    /// An `Option<&dyn ThumbnailCapability>` containing the thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Trait for thumbnail capability.
#[async_trait]
impl ThumbnailCapability for SvgFormatProvider {
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
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let svg_data = std::fs::read(&path_owned).map_err(crate::core::error::AppError::Io)?;

            let mut fontdb = usvg::fontdb::Database::new();
            fontdb.load_system_fonts();

            let opt = usvg::Options::default();
            let tree = usvg::Tree::from_data(&svg_data, &opt).map_err(|e| {
                crate::core::error::AppError::Generic(format!("SVG parse error: {}", e))
            })?;

            let size = tree.size();
            let width = size.width();
            let height = size.height();

            if width == 0.0 || height == 0.0 {
                return Err(crate::core::error::AppError::Generic(
                    "Invalid SVG dimensions".to_string(),
                ));
            }

            let scale = if width > height {
                size_hint as f32 / width
            } else {
                size_hint as f32 / height
            };

            let transform = tiny_skia::Transform::from_scale(scale, scale);
            let target_width = (width * scale).ceil() as u32;
            let target_height = (height * scale).ceil() as u32;

            let mut pixmap = Pixmap::new(target_width, target_height).ok_or_else(|| {
                crate::core::error::AppError::Generic("Failed to create pixmap buffer".to_string())
            })?;

            resvg::render(&tree, transform, &mut pixmap.as_mut());

            let encoder = webp::Encoder::from_rgba(pixmap.data(), target_width, target_height);
            let webp_data = encoder.encode(80.0);

            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
