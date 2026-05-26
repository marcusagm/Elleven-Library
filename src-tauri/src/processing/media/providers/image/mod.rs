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
