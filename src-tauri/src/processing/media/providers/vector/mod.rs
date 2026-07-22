//! Per-format vector providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct vector/postscript format and delegates all
//! extraction logic to the shared functions in
//! `crate::processing::media::extractors`.

pub mod postscript_format;
pub mod svg_format;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all vector format providers into a single vector.
///
/// This function is the single point of registration for all vector providers.
/// New vector formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All vector format providers.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(postscript_format::PostscriptFormatProvider::new()),
        Arc::new(svg_format::SvgFormatProvider::new()),
    ]
}
