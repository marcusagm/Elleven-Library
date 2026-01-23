mod database;
mod indexer;
mod protocols;
mod thumbnail_worker;
mod thumbnails;

use crate::database::Db;
use crate::indexer::Indexer;
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
async fn start_indexing(path: String, app: tauri::AppHandle) -> Result<(), String> {
    println!("COMMAND: start_indexing called with path: {}", path);

    // Get DB from state with safety
    let db = match app.try_state::<std::sync::Arc<Db>>() {
        Some(db) => db,
        None => return Err("Database not initialized".to_string()),
    };

    let indexer = Indexer::new(app.clone(), db.inner());

    let root = PathBuf::from(path);
    indexer.start_scan(root).await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Resolve paths
            let app_data = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).ok();

            let db_path = app_data.join("elleven.db");
            let thumbnails_dir = app_data.join("thumbnails");
            std::fs::create_dir_all(&thumbnails_dir).ok();

            // Initialize DB and Worker
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match Db::new(db_path).await {
                    Ok(db) => {
                        let db_arc = std::sync::Arc::new(db);
                        handle.manage(db_arc.clone());

                        let worker = crate::thumbnail_worker::ThumbnailWorker::new(
                            db_arc,
                            thumbnails_dir,
                            handle.clone(),
                        );
                        worker.start().await;
                    }
                    Err(e) => eprintln!("Failed to initialize database: {}", e),
                }
            });

            Ok(())
        })
        .register_uri_scheme_protocol("thumb", move |ctx, request| {
            crate::protocols::thumb_handler(ctx.app_handle(), &request)
        })
        .register_uri_scheme_protocol("orig", move |ctx, request| {
            crate::protocols::orig_handler(ctx.app_handle(), &request)
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![start_indexing])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
