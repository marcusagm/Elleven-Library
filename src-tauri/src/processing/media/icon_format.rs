use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use resvg::usvg;
use std::path::Path;
use tiny_skia::Pixmap;

/// Provider for generic file icons as a fallback
#[derive(Default)]
pub struct IconFormatProvider;

/// Implementation of `IconFormatProvider`.
impl IconFormatProvider {
    /// Create a new instance of `IconFormatProvider`.
    ///
    /// # Returns
    ///
    /// A new instance of `IconFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// SVG template for generic file icons.
const SVG_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 500">
  <path d="M 382.426 17.574 C 392.734 27.882 399.563 44.15 400 60 L 400 440 C 399.564 455.85 392.735 472.118 382.426 482.426 C 372.118 492.734 355.85 499.563 340 500 L 60 500 C 44.15 499.564 27.882 492.735 17.574 482.426 C 7.266 472.118 0.437 455.85 0 440 L 0 60 C 0.436 44.15 7.265 27.882 17.574 17.574 C 27.882 7.266 44.15 0.437 60 0 L 340 0 C 355.85 0.436 372.118 7.265 382.426 17.574 Z" style="fill: rgb(0, 160, 169);"/>
  <text style="fill: #ffffff; font-family: sans-serif; font-weight: bold; font-size: 80px; text-anchor: middle;" x="200" y="440">.generic</text>
</svg>"#;

/// Trait for format provider.
impl FormatProvider for IconFormatProvider {
    /// Get the name of the format provider.
    ///
    /// # Returns
    ///
    /// A `&'static str` containing the name of the format provider.
    fn name(&self) -> &'static str {
        "GENERIC_ICON_PROVIDER"
    }

    /// Get the supported extensions for the format.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec![] // Handled as fallback by registry
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "Generic Icon",
                vec!["generic"],
                vec!["image/svg+xml"],
                MediaType::Unknown,
                PreviewStrategy::BrowserNative,
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
}

/// Trait for thumbnail capability.
#[async_trait]
impl ThumbnailCapability for IconFormatProvider {
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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("???")
            .to_lowercase();

        tokio::task::spawn_blocking(move || {
            let svg_content = SVG_TEMPLATE.replace(".generic", &format!(".{}", ext));

            let opt = usvg::Options::default();
            let mut fontdb = usvg::fontdb::Database::new();
            fontdb.load_system_fonts();

            let tree = usvg::Tree::from_str(&svg_content, &opt)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

            let mut pixmap = Pixmap::new(size_hint, size_hint).ok_or_else(|| {
                crate::core::error::AppError::Generic("Pixmap failure".to_string())
            })?;

            let size = tree.size();
            let scale = (size_hint as f32 / size.width()).min(size_hint as f32 / size.height());
            let transform = tiny_skia::Transform::from_scale(scale, scale);

            resvg::render(&tree, transform, &mut pixmap.as_mut());

            let encoder = webp::Encoder::from_rgba(pixmap.data(), size_hint, size_hint);
            Ok(encoder.encode(85.0).to_vec())
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
