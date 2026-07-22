//! Per-format font providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct font format and delegates all extraction
//! logic to the shared functions in `crate::processing::media::extractors`.

pub mod otf;
pub mod ttc;
pub mod ttf;
pub mod woff;
pub mod woff2;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all font format providers into a single vector.
///
/// This function is the single point of registration for all font providers.
/// New font formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All font format providers.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(otf::OpenTypeFontProvider::new()),
        Arc::new(ttf::TrueTypeFontProvider::new()),
        Arc::new(ttc::TrueTypeCollectionProvider::new()),
        Arc::new(woff::WoffFontProvider::new()),
        Arc::new(woff2::Woff2FontProvider::new()),
    ]
}
