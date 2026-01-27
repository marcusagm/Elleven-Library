use crate::metadata_reader;
use std::collections::HashMap;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_image_exif(path: String) -> Result<HashMap<String, String>, String> {
    // Check if file exists
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("File not found".to_string());
    }

    // Run on blocking thread since file I/O can be slow
    let res = tauri::async_runtime::spawn_blocking(move || metadata_reader::read_exif(&path_buf))
        .await
        .map_err(|e| e.to_string())?;

    Ok(res)
}
