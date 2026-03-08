//! Format Registry and Capability definitions.
//!
//! This module provides the infrastructure for multi-format support in Mundam.
//! It defines traits for metadata extraction and thumbnail generation (Capabilities),
//! and a central Registry for O(1) format resolution.

pub mod capabilities;
pub mod provider;
pub mod registry;

pub use registry::FormatRegistry;

use crate::processing::media::affinity_format::AffinityFormatProvider;
use crate::processing::media::archive_format::ArchiveFormatProvider;
use crate::processing::media::audio_format::AudioFormatProvider;
use crate::processing::media::fallback_format::GenericByteFallbackProvider;
use crate::processing::media::font_format::FontFormatProvider;
use crate::processing::media::icon_format::IconFormatProvider;
use crate::processing::media::image_format::ImageFormatProvider;
use crate::processing::media::modern_image_format::ModernImageFormatProvider;
use crate::processing::media::pdf_format::PdfFormatProvider;
use crate::processing::media::raw_format::RawFormatProvider;
use crate::processing::media::svg_format::SvgFormatProvider;
use crate::processing::media::video_format::VideoFormatProvider;
use std::sync::Arc;

/// Factory function to build the main FormatRegistry.
///
/// This is used during application boot to register all supported format plugins.
pub fn build_format_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();

    // Register primary media providers
    registry.register(Arc::new(ImageFormatProvider::new()));
    registry.register(Arc::new(VideoFormatProvider::new()));
    registry.register(Arc::new(AudioFormatProvider::new()));
    registry.register(Arc::new(ModernImageFormatProvider::new()));
    registry.register(Arc::new(RawFormatProvider::new()));
    registry.register(Arc::new(ArchiveFormatProvider::new()));
    registry.register(Arc::new(AffinityFormatProvider::new()));
    registry.register(Arc::new(SvgFormatProvider::new()));
    registry.register(Arc::new(PdfFormatProvider::new()));
    registry.register(Arc::new(FontFormatProvider::new()));

    // Register generic fallbacks
    registry.register(Arc::new(IconFormatProvider::new()));
    registry.register(Arc::new(GenericByteFallbackProvider::new()));

    registry
}
