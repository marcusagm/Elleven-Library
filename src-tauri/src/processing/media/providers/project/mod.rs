//! Per-format project file providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct creative application project format (e.g. PSD,
//! Krita, Affinity) and delegates all extraction logic to the shared functions
//! in `crate::processing::media::extractors`.

pub mod affinity;
pub mod aseprite;
pub mod clipstudio;
pub mod coreldraw;
pub mod corelpainter;
pub mod figma;
pub mod gimp;
pub mod illustrator;
pub mod krita;
pub mod medibang;
pub mod painttoolsai;
pub mod penpot;
pub mod photoshop;
pub mod rebelle;
pub mod sketch;
pub mod xmind;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all project format providers into a single vector.
///
/// This function is the single point of registration for all project providers.
/// New project formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All project format providers.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(affinity::AffinityFormatProvider::new()),
        Arc::new(aseprite::AsepriteFormatProvider::new()),
        Arc::new(clipstudio::ClipStudioFormatProvider::new()),
        Arc::new(coreldraw::CoreldrawFormatProvider::new()),
        Arc::new(corelpainter::CorelPainterFormatProvider::new()),
        Arc::new(figma::FigmaFormatProvider::new()),
        Arc::new(illustrator::IllustratorFormatProvider::new()),
        Arc::new(gimp::GimpFormatProvider::new()),
        Arc::new(krita::KritaFormatProvider::new()),
        Arc::new(medibang::MedibangFormatProvider::new()),
        Arc::new(painttoolsai::PaintToolSaiFormatProvider::new()),
        Arc::new(penpot::PenpotFormatProvider::new()),
        Arc::new(photoshop::PhotoshopFormatProvider::new()),
        Arc::new(rebelle::RebelleFormatProvider::new()),
        Arc::new(sketch::SketchFormatProvider::new()),
        Arc::new(xmind::XMindFormatProvider::new()),
    ]
}
