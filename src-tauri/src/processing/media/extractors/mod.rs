//! Extractors for media files

pub mod ai;
pub mod aseprite;
pub mod binary_jpeg;
pub mod clip;
pub mod coreldraw;
pub mod corelpainter;
pub mod eps;
pub mod figma;
pub mod font;
pub mod gpr;
pub mod image;
pub mod mdp;
pub mod pdf;
pub mod penpot;
pub mod rebelle;
pub mod sai;
pub mod sai2;
pub mod sketch;
pub mod xcf;
pub mod x3f;
pub mod kdc;
pub mod jxl;
pub mod video;
pub mod audio;

// Convenience re-exports
pub use ai::{extract_ai_metadata, extract_ai_preview};
pub use aseprite::extract_aseprite_preview;
pub use binary_jpeg::extract_any_embedded;
pub use clip::{extract_clip_metadata, extract_clip_preview};
pub use coreldraw::{
    extract_coreldraw_dimensions, extract_coreldraw_metadata, extract_coreldraw_preview,
    extract_coreldraw_preview_highres, get_cdr_version_string,
};
pub use corelpainter::{extract_corel_painter_preview, extract_corelpainter_metadata};
pub use eps::{extract_eps_metadata, extract_eps_ps_preview};
pub use figma::{extract_figma_metadata, extract_figma_preview};
pub use font::{extract_font_metadata, generate_font_thumbnail};
pub use gpr::{extract_gpr_metadata, extract_gpr_preview, generate_gpr_thumbnail};
pub use image::{
    extract_exr_metadata, extract_ffmpeg_image_metadata, extract_raster_metadata,
    extract_raw_metadata, extract_raw_preview, generate_ffmpeg_image_preview,
    generate_ffmpeg_image_thumbnail, generate_hdr_exr_dds_preview, generate_hdr_exr_dds_thumbnail,
    generate_raster_preview, generate_raster_thumbnail, generate_raw_thumbnail,
    process_and_encode_webp,
};
pub use mdp::{extract_mdp_metadata, extract_mdp_preview};
pub use pdf::{extract_pdf_metadata, render_pdf_to_png};
pub use penpot::{extract_penpot_metadata, extract_penpot_preview};
pub use rebelle::extract_rebelle_preview;
pub use sai::{extract_sai_dimensions, extract_sai_metadata, extract_sai_preview};
pub use sai2::{extract_sai2_dimensions, extract_sai2_metadata, extract_sai2_preview};
pub use sketch::{extract_sketch_metadata, extract_sketch_preview};
pub use xcf::{extract_xcf_metadata, extract_xcf_preview};
pub use x3f::{extract_x3f_metadata, extract_x3f_preview, generate_x3f_thumbnail};
pub use kdc::{extract_kdc_metadata, extract_kdc_preview, generate_kdc_thumbnail};
pub use jxl::{extract_jxl_metadata, extract_jxl_preview, generate_jxl_thumbnail};
pub use video::{extract_video_technical_metadata, generate_video_thumbnail};
pub use audio::extract_audio_technical_metadata;

