//! Per-format 3D model providers for the Mundam media processing pipeline.
//!
//! Each module handles a distinct 3D model format (or a family of related
//! formats) and delegates extraction logic to the shared functions in
//! `crate::processing::media::extractors::model3d`.
//!
//! # Organisation
//!
//! | Category              | Modules                                          |
//! |-----------------------|--------------------------------------------------|
//! | DCC Projects          | `blender`                                        |
//! | Web-native 3D         | `gltf`                                           |
//! | Interchange formats   | `assimp_model` (FBX, OBJ, Collada, STL, 3DS, 3MF)|
//! | Scene descriptions    | `usd`                                            |
//! | Engineering CAD       | `cad` (STEP, IGES), `autocad` (DWG, DXF)         |

pub mod assimp_model;
pub mod autocad;
pub mod blender;
pub mod cad;
pub mod gltf;
pub mod usd;

use crate::core::formats::provider::FormatProvider;
use std::sync::Arc;

/// Collects all 3D model format providers into a single vector.
///
/// This function is the single point of registration for all 3D model providers.
/// New 3D model formats should add their provider instance here after declaring
/// the corresponding submodule above.
///
/// # Returns
///
/// All 3D model format providers, ordered by specificity: native formats first,
/// then interchange formats, and finally engineering CAD formats.
pub fn collect_providers() -> Vec<Arc<dyn FormatProvider>> {
    vec![
        Arc::new(blender::BlenderFormatProvider::new()),
        Arc::new(gltf::GltfFormatProvider::new()),
        Arc::new(assimp_model::AssimpModelProvider::new()),
        Arc::new(usd::UsdFormatProvider::new()),
        Arc::new(cad::CadFormatProvider::new()),
        Arc::new(autocad::AutocadFormatProvider::new()),
    ]
}
