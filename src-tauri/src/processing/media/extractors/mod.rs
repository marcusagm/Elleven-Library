//! Extractors for media files

pub mod ai;
pub mod aseprite;
pub mod binary_jpeg;
pub mod clip;
pub mod corel_painter;
pub mod coreldraw;
pub mod eps;
pub mod mdp;
pub mod penpot;
pub mod rebelle;
pub mod sai;
pub mod sai2;
pub mod sketch;
pub mod xcf;

// Convenience re-exports
pub use ai::extract_ai_preview;
pub use aseprite::extract_aseprite_preview;
pub use binary_jpeg::extract_any_embedded;
pub use clip::extract_clip_preview;
pub use corel_painter::extract_corel_painter_preview;
pub use coreldraw::{extract_coreldraw_preview, extract_coreldraw_preview_highres, extract_coreldraw_dimensions, get_cdr_version_string};
pub use eps::extract_eps_ps_preview;
pub use mdp::extract_mdp_preview;
pub use penpot::extract_penpot_preview;
pub use rebelle::extract_rebelle_preview;
pub use sai::{extract_sai_preview, extract_sai_dimensions};
pub use sai2::{extract_sai2_preview, extract_sai2_dimensions};
pub use sketch::extract_sketch_preview;
pub use xcf::{extract_xcf_preview, extract_xcf_dimensions};
