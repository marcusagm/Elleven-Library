//! Extractors for media files

pub mod ai;
pub mod aseprite;
pub mod binary_jpeg;
pub mod clip;
pub mod corelpainter;
pub mod coreldraw;
pub mod eps;
pub mod figma;
pub mod mdp;
pub mod penpot;
pub mod rebelle;
pub mod sai;
pub mod sai2;
pub mod sketch;
pub mod pdf;
pub mod xcf;

// Convenience re-exports
pub use ai::{extract_ai_preview, extract_ai_metadata};
pub use aseprite::extract_aseprite_preview;
pub use binary_jpeg::extract_any_embedded;
pub use clip::{extract_clip_metadata, extract_clip_preview};
pub use corelpainter::{extract_corel_painter_preview, extract_corelpainter_metadata};
pub use coreldraw::{
    extract_coreldraw_dimensions, extract_coreldraw_metadata, extract_coreldraw_preview,
    extract_coreldraw_preview_highres, get_cdr_version_string,
};
pub use eps::{extract_eps_ps_preview, extract_eps_metadata};
pub use figma::{extract_figma_metadata, extract_figma_preview};
pub use mdp::{extract_mdp_metadata, extract_mdp_preview};
pub use pdf::{render_pdf_to_png, extract_pdf_metadata};
pub use penpot::{extract_penpot_metadata, extract_penpot_preview};
pub use rebelle::extract_rebelle_preview;
pub use sai::{extract_sai_dimensions, extract_sai_metadata, extract_sai_preview};
pub use sai2::{extract_sai2_dimensions, extract_sai2_metadata, extract_sai2_preview};
pub use sketch::{extract_sketch_metadata, extract_sketch_preview};
pub use xcf::{extract_xcf_metadata, extract_xcf_preview};
