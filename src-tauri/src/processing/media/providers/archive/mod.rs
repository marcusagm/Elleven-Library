//! Per-format archive providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct archive container format and delegates
//! thumbnail extraction to the shared functions in
//! `crate::processing::media::extractors::archive`.
//!
//! # Organisation
//!
//! | Category              | Modules                                          |
//! |-----------------------|--------------------------------------------------|
//! | ZIP-based             | `zip_archive`, `comic_book_zip`                  |
//! | Non-extractable       | `compressed_archive` (RAR, 7z, TAR, GZIP)        |

pub mod comic_book_zip;
pub mod compressed_archive;
pub mod zip_archive;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all archive format providers into a single vector.
///
/// This function is the single point of registration for all archive providers.
/// New archive formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All archive format providers, ordered with extractable formats first.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(zip_archive::ZipArchiveProvider::new()),
        Arc::new(comic_book_zip::ComicBookZipProvider::new()),
        Arc::new(compressed_archive::CompressedArchiveProvider::new()),
    ]
}
