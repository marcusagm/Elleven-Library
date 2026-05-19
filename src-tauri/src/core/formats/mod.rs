//! Format Registry and Capability definitions.
//!
//! This module provides the infrastructure for multi-format support in Mundam.
//! It defines traits for metadata extraction and thumbnail generation (Capabilities),
//! and a central Registry for O(1) format resolution.

pub mod capabilities;
pub mod provider;
pub mod registry;
pub mod types;

pub use provider::SupportedFormat;
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

// Final Parity Imports
use crate::processing::media::providers::vector::postscript_format::PostscriptFormatProvider;
// use crate::processing::media::aseprite_format::AsepriteFormatProvider; // Removed legacy provider
use crate::processing::media::cad_format::CadFormatProvider;
use crate::processing::media::exr_format::ExrFormatProvider;
use crate::processing::media::model3d_format::Model3dFormatProvider;
use crate::processing::media::text_format::TextFormatProvider;
use crate::processing::media::usd_format::UsdFormatProvider;

// Project Providers
use crate::processing::media::providers::project::aseprite::AsepriteFormatProvider;
use crate::processing::media::providers::project::clipstudio::ClipStudioFormatProvider;
use crate::processing::media::providers::project::coreldraw::CoreldrawFormatProvider;
use crate::processing::media::providers::project::corelpainter::CorelPainterFormatProvider;
use crate::processing::media::providers::project::figma::FigmaFormatProvider;
use crate::processing::media::providers::project::gimp::GimpFormatProvider;
use crate::processing::media::providers::project::krita::KritaFormatProvider;
use crate::processing::media::providers::project::medibang::MedibangFormatProvider;
use crate::processing::media::providers::project::painttoolsai::PaintToolSaiFormatProvider;
use crate::processing::media::providers::project::penpot::PenpotFormatProvider;
use crate::processing::media::providers::project::photoshop::PhotoshopFormatProvider;
use crate::processing::media::providers::project::illustrator::IllustratorFormatProvider;
use crate::processing::media::providers::project::rebelle::RebelleFormatProvider;
use crate::processing::media::providers::project::sketch::SketchFormatProvider;
use crate::processing::media::providers::project::xmind::XMindFormatProvider;

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
    registry.register(Arc::new(TextFormatProvider::new()));

    // Register Final Parity Providers
    registry.register(Arc::new(PostscriptFormatProvider::new()));
    registry.register(Arc::new(ExrFormatProvider::new()));
    registry.register(Arc::new(Model3dFormatProvider::new()));
    registry.register(Arc::new(UsdFormatProvider::new()));
    registry.register(Arc::new(CadFormatProvider::new()));

    //Register Project Providers
    registry.register(Arc::new(AsepriteFormatProvider::new()));
    registry.register(Arc::new(ClipStudioFormatProvider::new()));
    registry.register(Arc::new(CoreldrawFormatProvider::new()));
    registry.register(Arc::new(CorelPainterFormatProvider::new()));
    registry.register(Arc::new(FigmaFormatProvider::new()));
    registry.register(Arc::new(GimpFormatProvider::new()));
    registry.register(Arc::new(KritaFormatProvider::new()));
    registry.register(Arc::new(MedibangFormatProvider::new()));
    registry.register(Arc::new(PaintToolSaiFormatProvider::new()));
    registry.register(Arc::new(PenpotFormatProvider::new()));
    registry.register(Arc::new(IllustratorFormatProvider::new()));
    registry.register(Arc::new(PhotoshopFormatProvider::new()));
    registry.register(Arc::new(RebelleFormatProvider::new()));
    registry.register(Arc::new(SketchFormatProvider::new()));
    registry.register(Arc::new(XMindFormatProvider::new()));

    // Register generic fallbacks
    registry.register(Arc::new(IconFormatProvider::new()));
    registry.register(Arc::new(GenericByteFallbackProvider::new()));

    registry
}
