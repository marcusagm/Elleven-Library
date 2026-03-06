#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod core;
pub mod db;
pub mod delivery;
pub mod feature;
mod indexer;
pub mod infra;
// Moved to media: metadata_reader, ffmpeg
mod protocols;
// Moved to thumbnails: thumbnail_worker, thumbnail_priority
pub mod formats;
pub mod lifecycle;
mod thumbnails;
// Moved to settings: config
pub mod library;
mod media;
mod settings;
mod streaming;
mod transcoding;

use crate::core::events::AppEventBus;
use crate::db::Db;
use crate::indexer::Indexer;
use crate::infra::events::TokioEventBus;
use crate::lifecycle::LifecycleRegistry;
use std::sync::Arc;
use tauri::Manager;

/// Holds the session token used to authenticate streaming server requests.
///
/// This token is generated once at app boot (UUID v4) and shared with
/// the frontend via the `get_streaming_token` Tauri command.
pub struct StreamingSessionToken(pub String);

/// Returns the streaming session token to the frontend.
///
/// The frontend must include this token as a `?token=xxx` query parameter
/// on every request to the embedded HLS streaming server.
#[tauri::command]
fn get_streaming_token(token_state: tauri::State<'_, StreamingSessionToken>) -> String {
    token_state.0.clone()
}

#[allow(clippy::expect_used)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured tracing
    crate::infra::telemetry::init_telemetry();

    let builder = tauri::Builder::default();
    crate::protocols::register_all(builder)
        .setup(|app| {
            // Resolve paths
            let app_data = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).ok();

            let db_path = app_data.join("mundam.db");
            let thumbnails_dir = app_data.join("thumbnails");
            std::fs::create_dir_all(&thumbnails_dir).ok();

            // Initialize Event Bus (System Nervous System)
            let event_bus = Arc::new(TokioEventBus::new());
            app.manage(event_bus.clone() as Arc<dyn AppEventBus>);

            // Create the lifecycle registry — central hub for managing all background tasks
            let lifecycle = std::sync::Arc::new(LifecycleRegistry::new());
            app.manage(lifecycle.clone());

            // Generate a session token for streaming server authentication
            let session_token = uuid::Uuid::new_v4().to_string();
            app.manage(StreamingSessionToken(session_token.clone()));

            // Initialize DB and Worker
            let handle = app.handle().clone();
            let lifecycle_for_setup = lifecycle.clone();
            let streaming_session_token = session_token;
            tauri::async_runtime::spawn(async move {
                // Initialize V2 Database Infrastructure
                let v2_db_manager =
                    match crate::infra::database::manager::DbManager::new(&db_path).await {
                        Ok(manager) => manager,
                        Err(err) => {
                            tracing::error!("Failed to initialize V2 database manager: {}", err);
                            return;
                        }
                    };
                let asset_query_handler =
                    Arc::new(crate::infra::database::queries::SqliteAssetQueries::new(
                        v2_db_manager.pool().clone(),
                    ));
                handle.manage(
                    asset_query_handler as Arc<dyn crate::core::repository::AssetQueryHandler>,
                );

                // Initialize Asset Ledger (Real SQLx Adapter)
                let asset_ledger =
                    Arc::new(crate::infra::database::ledger::SqliteAssetLedger::new(
                        v2_db_manager.pool().clone(),
                        event_bus.clone(),
                    ));
                handle.manage(asset_ledger.clone()
                    as Arc<dyn crate::core::ledger::port::TransactionalAssetLedger>);

                match Db::new(db_path).await {
                    Ok(db) => {
                        let db_arc = std::sync::Arc::new(db);
                        let watcher_registry = std::sync::Arc::new(tokio::sync::Mutex::new(
                            crate::indexer::WatcherRegistry::default(),
                        ));

                        // Load Config
                        let app_config = crate::settings::config::load_config(&db_arc).await;
                        let config_state = crate::settings::config::ConfigState(
                            std::sync::Mutex::new(app_config.clone()),
                        );

                        let priority_state = std::sync::Arc::new(
                            crate::thumbnails::priority::ThumbnailPriorityState::default(),
                        );

                        handle.manage(db_arc.clone());
                        handle.manage(watcher_registry.clone());
                        handle.manage(config_state);
                        handle.manage(priority_state.clone());

                        // Start Thumbnail Worker with lifecycle token
                        let thumbnail_token = lifecycle_for_setup.child_token();
                        let worker = crate::thumbnails::worker::ThumbnailWorker::new(
                            db_arc.clone(),
                            thumbnails_dir,
                            handle.clone(),
                            app_config,
                            priority_state,
                        );
                        let thumbnail_handle = worker.start(thumbnail_token.clone());
                        lifecycle_for_setup.register(
                            "thumbnail_worker".to_string(),
                            thumbnail_token,
                            thumbnail_handle,
                        );

                        // Start Watchers for Existing Roots
                        if let Ok(roots) = db_arc.get_all_root_folders().await {
                            tracing::info!("Starting watchers for {} roots", roots.len());
                            for (_id, path) in roots {
                                let indexer = Indexer::new(
                                    handle.clone(),
                                    &db_arc,
                                    watcher_registry.clone(),
                                    lifecycle_for_setup.clone(),
                                    asset_ledger.clone(),
                                );
                                let root_path = std::path::PathBuf::from(path);
                                indexer.start_scan(root_path).await;
                            }
                        }

                        // Start HLS Streaming Server with lifecycle token
                        // Started after DB init because it needs Arc<Db> for path scope validation
                        let streaming_token = lifecycle_for_setup.child_token();
                        let streaming_handle = crate::streaming::server::spawn_server(
                            handle.clone(),
                            streaming_token.clone(),
                            db_arc.clone(),
                            streaming_session_token,
                        );
                        lifecycle_for_setup.register(
                            "streaming_server".to_string(),
                            streaming_token,
                            streaming_handle,
                        );
                    }
                    Err(db_error) => tracing::error!("Failed to initialize database: {}", db_error),
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_mcp_bridge::init())
        .invoke_handler(tauri::generate_handler![
            library::commands::indexing::start_indexing,
            library::commands::tags::create_tag,
            library::commands::tags::update_tag,
            library::commands::tags::delete_tag,
            library::commands::tags::get_all_tags,
            library::commands::tags::get_library_stats,
            library::commands::tags::add_tag_to_asset,
            library::commands::tags::remove_tag_from_asset,
            library::commands::tags::get_tags_for_asset,
            library::commands::tags::add_tags_to_assets_batch,
            library::commands::tags::remove_tags_from_assets_batch,
            library::commands::tags::replace_tags_for_assets_batch,
            library::commands::tags::get_assets_filtered,
            library::commands::tags::get_asset_count_filtered,
            library::commands::tags::update_asset_rating,
            library::commands::tags::update_asset_notes,
            library::commands::metadata::get_asset_exif,
            thumbnails::commands::request_thumbnail_regenerate,
            thumbnails::commands::set_thumbnail_priority,
            library::commands::folders::add_location,
            library::commands::folders::remove_location,
            library::commands::folders::get_locations,
            library::commands::folders::get_all_subfolders,
            library::commands::folders::get_subfolder_counts,
            library::commands::folders::get_location_root_counts,
            library::commands::smart_folders::get_smart_folders,
            library::commands::smart_folders::save_smart_folder,
            library::commands::smart_folders::update_smart_folder,
            library::commands::smart_folders::delete_smart_folder,
            settings::commands::get_setting,
            settings::commands::set_setting,
            settings::commands::run_db_maintenance,
            settings::commands::send_telemetry_log,
            library::commands::formats::get_library_supported_formats,
            media::commands::get_audio_waveform_data,
            // Transcoding commands
            transcoding::commands::needs_transcoding,
            transcoding::commands::is_native_format,
            transcoding::commands::get_stream_url,
            transcoding::commands::get_quality_options,
            transcoding::commands::transcode_file,
            transcoding::commands::is_cached,
            transcoding::commands::get_cache_stats,
            transcoding::commands::cleanup_cache,
            transcoding::commands::clear_cache,
            transcoding::commands::ffmpeg_available,
            // Streaming security
            get_streaming_token,
            // Color Analysis
            library::commands::colors::get_asset_colors,
            library::commands::colors::reextract_asset_colors,
            library::commands::colors::reextract_all_colors
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                // Ensure graceful shutdown of all background tasks
                if let Some(lifecycle) = app_handle.try_state::<std::sync::Arc<LifecycleRegistry>>()
                {
                    tracing::info!(
                        "Application exit requested. Starting graceful background shutdown."
                    );
                    tauri::async_runtime::block_on(async {
                        lifecycle.shutdown_all().await;
                    });
                }
            }
        });
}
