//! Per-format image providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct image format (or a tightly-related family of
//! aliases) and delegates all extraction logic to the shared functions in
//! `crate::processing::media::extractors::image`. This keeps the provider
//! layer thin and ensures that bug-fixes to the core extraction pipeline
//! benefit every format simultaneously.
//!
//! # Organisation
//!
//! | Category      | Modules                                                  |
//! |---------------|----------------------------------------------------------|
//! | Standard raster | `jpeg`, `png`, `gif`, `bmp`, `tiff`, `webp`, `tga`  |
//! | Compressed    | `hdr`, `dds`, `netpbm`                                   |
//! | HDR / VFX     | `exr`                                                    |
//! | Modern        | `heic`, `avif`, `jxl`                                    |
//! | RAW — Canon   | `canon`                                                  |
//! | RAW — Nikon   | `nikon`                                                  |
//! | RAW — Sony    | `sony`                                                   |
//! | RAW — Fujifilm| `fujifilm`                                               |
//! | RAW — other   | `dng`, `olympus`, `panasonic`, `pentax`, `samsung`,      |
//! |               | `sigma`, `hasselblad`, `kodak`, `phaseone`, `leaf`,      |
//! |               | `leica`, `minolta`, `mamiya`, `gopro`, `epson`,          |
//! |               | `generic_raw`                                            |

pub mod avif;
pub mod bmp;
pub mod canon;
pub mod cur;
pub mod dds;
pub mod dng;
pub mod epson;
pub mod exr;
pub mod fujifilm;
pub mod generic_raw;
pub mod gif;
pub mod gopro;
pub mod hasselblad;
pub mod hdr;
pub mod heic;
pub mod icns;
pub mod ico;
pub mod jpeg;
pub mod jxl;
pub mod kodak;
pub mod leaf;
pub mod leica;
pub mod mamiya;
pub mod minolta;
pub mod netpbm;
pub mod nikon;
pub mod olympus;
pub mod panasonic;
pub mod pentax;
pub mod phaseone;
pub mod png;
pub mod samsung;
pub mod sigma;
pub mod sony;
pub mod tga;
pub mod tiff;
pub mod webp;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all image format providers into a single vector.
///
/// This function is the single point of registration for all image providers.
/// New image formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All image format providers, ordered by specificity: standard raster first,
/// then HDR/VFX, modern codecs, and finally RAW photography providers.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        // Standard Raster
        Arc::new(bmp::BmpFormatProvider::new()),
        Arc::new(cur::CurFormatProvider::new()),
        Arc::new(dds::DdsFormatProvider::new()),
        Arc::new(gif::GifFormatProvider::new()),
        Arc::new(hdr::HdrFormatProvider::new()),
        Arc::new(ico::IcoFormatProvider::new()),
        Arc::new(icns::IcnsFormatProvider::new()),
        Arc::new(jpeg::JpegFormatProvider::new()),
        Arc::new(netpbm::NetpbmFormatProvider::new()),
        Arc::new(png::PngFormatProvider::new()),
        Arc::new(tga::TgaFormatProvider::new()),
        Arc::new(tiff::TiffFormatProvider::new()),
        Arc::new(webp::WebpFormatProvider::new()),
        // HDR / VFX
        Arc::new(exr::ExrFormatProvider::new()),
        // Modern
        Arc::new(avif::AvifFormatProvider::new()),
        Arc::new(heic::HeicFormatProvider::new()),
        Arc::new(jxl::JxlFormatProvider::new()),
        // RAW Photography
        Arc::new(canon::CanonRawFormatProvider::new()),
        Arc::new(dng::DngFormatProvider::new()),
        Arc::new(epson::EpsonRawFormatProvider::new()),
        Arc::new(fujifilm::FujifilmRawFormatProvider::new()),
        Arc::new(generic_raw::GenericRawFormatProvider::new()),
        Arc::new(gopro::GoproRawFormatProvider::new()),
        Arc::new(hasselblad::HasselbladRawFormatProvider::new()),
        Arc::new(kodak::KodakRawFormatProvider::new()),
        Arc::new(leaf::LeafRawFormatProvider::new()),
        Arc::new(leica::LeicaRawFormatProvider::new()),
        Arc::new(mamiya::MamiyaRawFormatProvider::new()),
        Arc::new(minolta::MinoltaRawFormatProvider::new()),
        Arc::new(nikon::NikonRawFormatProvider::new()),
        Arc::new(olympus::OlympusRawFormatProvider::new()),
        Arc::new(panasonic::PanasonicRawFormatProvider::new()),
        Arc::new(pentax::PentaxRawFormatProvider::new()),
        Arc::new(phaseone::PhaseOneRawFormatProvider::new()),
        Arc::new(samsung::SamsungRawFormatProvider::new()),
        Arc::new(sigma::SigmaRawFormatProvider::new()),
        Arc::new(sony::SonyRawFormatProvider::new()),
    ]
}
