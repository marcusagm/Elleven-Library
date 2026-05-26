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

// Legacy providers (non-image categories, not yet migrated to providers/)
use crate::processing::media::archive_format::ArchiveFormatProvider;
use crate::processing::media::audio_format::AudioFormatProvider;
use crate::processing::media::cad_format::CadFormatProvider;
use crate::processing::media::fallback_format::GenericByteFallbackProvider;
use crate::processing::media::model3d_format::Model3dFormatProvider;
use crate::processing::media::text_format::TextFormatProvider;
use crate::processing::media::usd_format::UsdFormatProvider;
use crate::processing::media::video_format::VideoFormatProvider;

// Document Providers
use crate::processing::media::providers::document::pdf_format::PdfFormatProvider;

// Font Providers
use crate::processing::media::providers::font::otf::OpenTypeFontProvider;
use crate::processing::media::providers::font::ttc::TrueTypeCollectionProvider;
use crate::processing::media::providers::font::ttf::TrueTypeFontProvider;
use crate::processing::media::providers::font::woff::WoffFontProvider;
use crate::processing::media::providers::font::woff2::Woff2FontProvider;

// Image Providers — Standard Raster
use crate::processing::media::providers::image::bmp::BmpFormatProvider;
use crate::processing::media::providers::image::cur::CurFormatProvider;
use crate::processing::media::providers::image::dds::DdsFormatProvider;
use crate::processing::media::providers::image::gif::GifFormatProvider;
use crate::processing::media::providers::image::hdr::HdrFormatProvider;
use crate::processing::media::providers::image::icns::IcnsFormatProvider;
use crate::processing::media::providers::image::ico::IcoFormatProvider;
use crate::processing::media::providers::image::jpeg::JpegFormatProvider;
use crate::processing::media::providers::image::netpbm::NetpbmFormatProvider;
use crate::processing::media::providers::image::png::PngFormatProvider;
use crate::processing::media::providers::image::tga::TgaFormatProvider;
use crate::processing::media::providers::image::tiff::TiffFormatProvider;
use crate::processing::media::providers::image::webp::WebpFormatProvider;

// Image Providers — HDR / VFX
use crate::processing::media::providers::image::exr::ExrFormatProvider;

// Image Providers — Modern
use crate::processing::media::providers::image::avif::AvifFormatProvider;
use crate::processing::media::providers::image::heic::HeicFormatProvider;
use crate::processing::media::providers::image::jxl::JxlFormatProvider;

// Image Providers — RAW
use crate::processing::media::providers::image::canon::CanonRawFormatProvider;
use crate::processing::media::providers::image::dng::DngFormatProvider;
use crate::processing::media::providers::image::epson::EpsonRawFormatProvider;
use crate::processing::media::providers::image::fujifilm::FujifilmRawFormatProvider;
use crate::processing::media::providers::image::generic_raw::GenericRawFormatProvider;
use crate::processing::media::providers::image::gopro::GoproRawFormatProvider;
use crate::processing::media::providers::image::hasselblad::HasselbladRawFormatProvider;
use crate::processing::media::providers::image::kodak::KodakRawFormatProvider;
use crate::processing::media::providers::image::leaf::LeafRawFormatProvider;
use crate::processing::media::providers::image::leica::LeicaRawFormatProvider;
use crate::processing::media::providers::image::mamiya::MamiyaRawFormatProvider;
use crate::processing::media::providers::image::minolta::MinoltaRawFormatProvider;
use crate::processing::media::providers::image::nikon::NikonRawFormatProvider;
use crate::processing::media::providers::image::olympus::OlympusRawFormatProvider;
use crate::processing::media::providers::image::panasonic::PanasonicRawFormatProvider;
use crate::processing::media::providers::image::pentax::PentaxRawFormatProvider;
use crate::processing::media::providers::image::phaseone::PhaseOneRawFormatProvider;
use crate::processing::media::providers::image::samsung::SamsungRawFormatProvider;
use crate::processing::media::providers::image::sigma::SigmaRawFormatProvider;
use crate::processing::media::providers::image::sony::SonyRawFormatProvider;

// Project Providers
use crate::processing::media::providers::project::affinity::AffinityFormatProvider;
use crate::processing::media::providers::project::aseprite::AsepriteFormatProvider;
use crate::processing::media::providers::project::clipstudio::ClipStudioFormatProvider;
use crate::processing::media::providers::project::coreldraw::CoreldrawFormatProvider;
use crate::processing::media::providers::project::corelpainter::CorelPainterFormatProvider;
use crate::processing::media::providers::project::figma::FigmaFormatProvider;
use crate::processing::media::providers::project::gimp::GimpFormatProvider;
use crate::processing::media::providers::project::illustrator::IllustratorFormatProvider;
use crate::processing::media::providers::project::krita::KritaFormatProvider;
use crate::processing::media::providers::project::medibang::MedibangFormatProvider;
use crate::processing::media::providers::project::painttoolsai::PaintToolSaiFormatProvider;
use crate::processing::media::providers::project::penpot::PenpotFormatProvider;
use crate::processing::media::providers::project::photoshop::PhotoshopFormatProvider;
use crate::processing::media::providers::project::rebelle::RebelleFormatProvider;
use crate::processing::media::providers::project::sketch::SketchFormatProvider;
use crate::processing::media::providers::project::xmind::XMindFormatProvider;

// Vector Providers
use crate::processing::media::providers::vector::postscript_format::PostscriptFormatProvider;
use crate::processing::media::providers::vector::svg_format::SvgFormatProvider;

use std::sync::Arc;

/// Factory function to build the main `FormatRegistry`.
///
/// Registers every supported format provider during application boot. Providers
/// are evaluated in registration order when two providers match the same magic
/// bytes; more specific providers should be registered before generic fallbacks.
///
/// # Panics
///
/// This function does not panic. Individual registration failures are silently
/// ignored by the registry (duplicate extensions are overwritten).
pub fn build_format_registry() -> FormatRegistry {
    let mut registry = FormatRegistry::new();

    // Legacy non-image providers (not yet split into providers/)
    registry.register(Arc::new(VideoFormatProvider::new()));
    registry.register(Arc::new(AudioFormatProvider::new()));
    registry.register(Arc::new(ArchiveFormatProvider::new()));
    registry.register(Arc::new(TextFormatProvider::new()));
    registry.register(Arc::new(Model3dFormatProvider::new()));
    registry.register(Arc::new(UsdFormatProvider::new()));
    registry.register(Arc::new(CadFormatProvider::new()));

    // Document Providers
    registry.register(Arc::new(PdfFormatProvider::new()));

    // Font Providers
    registry.register(Arc::new(OpenTypeFontProvider::new()));
    registry.register(Arc::new(TrueTypeFontProvider::new()));
    registry.register(Arc::new(TrueTypeCollectionProvider::new()));
    registry.register(Arc::new(WoffFontProvider::new()));
    registry.register(Arc::new(Woff2FontProvider::new()));

    // Image Providers — Standard Raster
    registry.register(Arc::new(BmpFormatProvider::new()));
    registry.register(Arc::new(CurFormatProvider::new()));
    registry.register(Arc::new(DdsFormatProvider::new()));
    registry.register(Arc::new(GifFormatProvider::new()));
    registry.register(Arc::new(HdrFormatProvider::new()));
    registry.register(Arc::new(IcoFormatProvider::new()));
    registry.register(Arc::new(IcnsFormatProvider::new()));
    registry.register(Arc::new(JpegFormatProvider::new()));
    registry.register(Arc::new(NetpbmFormatProvider::new()));
    registry.register(Arc::new(PngFormatProvider::new()));
    registry.register(Arc::new(TgaFormatProvider::new()));
    registry.register(Arc::new(TiffFormatProvider::new()));
    registry.register(Arc::new(WebpFormatProvider::new()));

    // Image Providers — HDR / VFX
    registry.register(Arc::new(ExrFormatProvider::new()));

    // Image Providers — Modern
    registry.register(Arc::new(AvifFormatProvider::new()));
    registry.register(Arc::new(HeicFormatProvider::new()));
    registry.register(Arc::new(JxlFormatProvider::new()));

    // Image Providers — RAW Photography
    registry.register(Arc::new(CanonRawFormatProvider::new()));
    registry.register(Arc::new(DngFormatProvider::new()));
    registry.register(Arc::new(EpsonRawFormatProvider::new()));
    registry.register(Arc::new(FujifilmRawFormatProvider::new()));
    registry.register(Arc::new(GenericRawFormatProvider::new()));
    registry.register(Arc::new(GoproRawFormatProvider::new()));
    registry.register(Arc::new(HasselbladRawFormatProvider::new()));
    registry.register(Arc::new(KodakRawFormatProvider::new()));
    registry.register(Arc::new(LeafRawFormatProvider::new()));
    registry.register(Arc::new(LeicaRawFormatProvider::new()));
    registry.register(Arc::new(MamiyaRawFormatProvider::new()));
    registry.register(Arc::new(MinoltaRawFormatProvider::new()));
    registry.register(Arc::new(NikonRawFormatProvider::new()));
    registry.register(Arc::new(OlympusRawFormatProvider::new()));
    registry.register(Arc::new(PanasonicRawFormatProvider::new()));
    registry.register(Arc::new(PentaxRawFormatProvider::new()));
    registry.register(Arc::new(PhaseOneRawFormatProvider::new()));
    registry.register(Arc::new(SamsungRawFormatProvider::new()));
    registry.register(Arc::new(SigmaRawFormatProvider::new()));
    registry.register(Arc::new(SonyRawFormatProvider::new()));

    // Project Providers
    registry.register(Arc::new(AffinityFormatProvider::new()));
    registry.register(Arc::new(AsepriteFormatProvider::new()));
    registry.register(Arc::new(ClipStudioFormatProvider::new()));
    registry.register(Arc::new(CoreldrawFormatProvider::new()));
    registry.register(Arc::new(CorelPainterFormatProvider::new()));
    registry.register(Arc::new(FigmaFormatProvider::new()));
    registry.register(Arc::new(IllustratorFormatProvider::new()));
    registry.register(Arc::new(GimpFormatProvider::new()));
    registry.register(Arc::new(KritaFormatProvider::new()));
    registry.register(Arc::new(MedibangFormatProvider::new()));
    registry.register(Arc::new(PaintToolSaiFormatProvider::new()));
    registry.register(Arc::new(PenpotFormatProvider::new()));
    registry.register(Arc::new(PhotoshopFormatProvider::new()));
    registry.register(Arc::new(RebelleFormatProvider::new()));
    registry.register(Arc::new(SketchFormatProvider::new()));
    registry.register(Arc::new(XMindFormatProvider::new()));

    // Vector Providers
    registry.register(Arc::new(PostscriptFormatProvider::new()));
    registry.register(Arc::new(SvgFormatProvider::new()));

    // Generic fallbacks (must be last)
    registry.register(Arc::new(GenericByteFallbackProvider::new()));

    registry
}
