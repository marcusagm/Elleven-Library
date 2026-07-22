//! Per-format document providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct document format and delegates all extraction
//! logic to the shared functions in `crate::processing::media::extractors`.

pub mod pdf_format;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all document format providers into a single vector.
///
/// This function is the single point of registration for all document providers.
/// New document formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All document format providers.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(pdf_format::PdfFormatProvider::new()),
    ]
}
