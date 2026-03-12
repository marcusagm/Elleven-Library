use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
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
        vec!["svg", "svgz"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Scalable Vector Graphics",
                vec!["svg", "svgz"],
                vec!["image/svg+xml"],
                MediaType::Image,
                PreviewStrategy::BrowserNative,
                PlaybackStrategy::None,
            ),
        ]
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

            let target_size = tree.size().scale_to(usvg::Size::from_wh(size_hint as f32, size_hint as f32).unwrap());

            let transform = tiny_skia::Transform::from_scale(
                target_size.width() / tree.size().width(),
                target_size.height() / tree.size().height(),
            );

            let mut pixmap = Pixmap::new(target_size.width() as u32, target_size.height() as u32)
                .ok_or_else(|| {
                crate::core::error::AppError::Generic("Failed to create pixmap buffer".to_string())
            })?;

            resvg::render(&tree, transform, &mut pixmap.as_mut());

            // Use the shared helper from raw_format if possible, or encode directly
            // Since we already have a pixmap, let's just encode it to WebP
            let encoder = webp::Encoder::from_rgba(
                pixmap.data(),
                target_size.width() as u32,
                target_size.height() as u32,
            );
            let webp_data = encoder.encode(85.0); // Slightly higher quality for vectors

            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
