//! Provider registration hub for all media format categories.
//!
//! Each subcategory module (e.g. `image`, `audio`, `video`) exposes a
//! `collect_providers()` function that returns all providers for that category.
//! This module re-exports those subcategories and provides a top-level
//! `collect_all_providers()` that aggregates every provider across the entire
//! application for use during bootstrap.
//!
//! # Adding a new provider
//!
//! 1. Create the provider file under the appropriate subcategory
//!    (e.g. `providers/image/my_format.rs`).
//! 2. Declare it as `pub mod my_format;` in the subcategory `mod.rs`.
//! 3. Add `Arc::new(my_format::MyFormatProvider::new())` to the subcategory's
//!    `collect_providers()` function.
//!
//! That's it — no other files need to be changed.

pub mod archive;
pub mod audio;
pub mod document;
pub mod font;
pub mod image;
pub mod model3d;
pub mod project;
pub mod vector;
pub mod video;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all format providers from every category into a single vector.
///
/// Providers are returned in registration order: specific categories first,
/// generic fallback last (handled separately by `fallback_provider()`).
///
/// # Returns
///
/// A flat vector of every registered `FormatProvider`, ready for batch
/// insertion into a `FormatRegistry`.
pub fn collect_all_providers() -> Vec<Arc<dyn FormatProvider>> {
    let mut all_providers: Vec<Arc<dyn FormatProvider>> = Vec::new();

    all_providers.extend(archive::collect_providers());
    all_providers.extend(document::collect_providers());
    all_providers.extend(font::collect_providers());
    all_providers.extend(image::collect_providers());
    all_providers.extend(model3d::collect_providers());
    all_providers.extend(project::collect_providers());
    all_providers.extend(vector::collect_providers());
    all_providers.extend(video::collect_providers());
    all_providers.extend(audio::collect_providers());

    all_providers
}

/// Returns the generic byte fallback provider.
///
/// This provider must be registered **last** in the `FormatRegistry` so that
/// it only matches files that no specific provider could identify. It is kept
/// separate from `collect_all_providers()` to enforce this ordering constraint
/// at the call site.
///
/// # Returns
///
/// The `GenericByteFallbackProvider` wrapped in an `Arc`.
pub fn fallback_provider() -> Arc<dyn FormatProvider> {
    Arc::new(crate::processing::media::fallback_format::GenericByteFallbackProvider::new())
}
