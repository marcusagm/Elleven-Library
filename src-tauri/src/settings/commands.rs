use crate::core::error::AppResult;
use crate::db::Db;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_setting(
    key: String,
    db: State<'_, std::sync::Arc<Db>>,
) -> AppResult<Option<Value>> {
    Ok(db.get_setting(&key).await?)
}

#[tauri::command]
pub async fn set_setting(
    key: String,
    value: Value,
    db: State<'_, std::sync::Arc<Db>>,
) -> AppResult<()> {
    Ok(db.set_setting(&key, &value).await?)
}

#[tauri::command]
pub async fn run_db_maintenance(db: State<'_, std::sync::Arc<Db>>) -> AppResult<()> {
    db.run_maintenance().await
}

#[tauri::command]
pub fn send_telemetry_log(level: String, component: String, message: String) {
    match level.to_lowercase().as_str() {
        "error" => tracing::error!(component = %component, "{}", message),
        "warn" => tracing::warn!(component = %component, "{}", message),
        "debug" => tracing::debug!(component = %component, "{}", message),
        _ => tracing::info!(component = %component, "{}", message),
    }
}
