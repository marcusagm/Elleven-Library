use crate::db::models::AssetMetadata;
use crate::formats::FileFormat;
use chrono::{DateTime, Utc};
use imagesize::size;
use std::path::Path;

pub fn get_asset_metadata(path: &Path) -> Option<AssetMetadata> {
    // Attempt to detect format securely
    let file_format = FileFormat::detect(path)?;

    let metadata = std::fs::metadata(path).ok()?;
    let modified_at: DateTime<Utc> = metadata.modified().ok()?.into();
    let created_at: DateTime<Utc> = metadata
        .created()
        .ok()
        .map(|c| c.into())
        .unwrap_or(modified_at);

    let (width, height) = match size(path) {
        Ok(dim) => (Some(dim.width as i32), Some(dim.height as i32)),
        Err(_) => (None, None),
    };

    let filename = path.file_name()?.to_string_lossy().to_string();
    let format = file_format.extensions.first()?.to_string();
    let media_type = file_format.type_category.to_string();

    Some(AssetMetadata {
        id: 0,
        path: path.to_string_lossy().to_string(),
        filename,
        width,
        height,
        size: metadata.len() as i64,
        format,
        media_type,
        thumbnail_path: None,
        rating: 0,
        notes: None,
        modified_at,
        created_at,
        added_at: None,
    })
}
