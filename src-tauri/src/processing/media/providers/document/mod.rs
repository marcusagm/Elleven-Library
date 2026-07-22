//! Per-format document providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct document format and delegates all extraction
//! logic to the shared functions in `crate::processing::media::extractors`.
//!
//! # Organisation
//!
//! | Category              | Modules                                          |
//! |-----------------------|--------------------------------------------------|
//! | Portable documents    | `pdf_format`                                     |
//! | Plain text            | `plain_text`                                     |
//! | Rich text             | `markdown`                                       |
//! | Structured data       | `structured_data`                                |

pub mod markdown;
pub mod pdf_format;
pub mod plain_text;
pub mod structured_data;

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
        Arc::new(plain_text::PlainTextFormatProvider::new()),
        Arc::new(markdown::MarkdownFormatProvider::new()),
        Arc::new(structured_data::StructuredDataFormatProvider::new()),
    ]
}
