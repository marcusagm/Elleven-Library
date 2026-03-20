#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod core;
pub mod delivery;
pub mod feature;
pub mod infra;
pub mod lifecycle;
pub mod processing;
use crate::core::events::AppEventBus;
use crate::delivery::streaming::server::start_server;
use crate::feature::transcoding::cache::TranscodeCache;
use crate::infra::events::TokioEventBus;
use crate::lifecycle::LifecycleRegistry;
use std::sync::Arc;
use tauri::{Emitter, Manager};

/// Runs the application.
#[allow(clippy::expect_used)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize structured tracing
    crate::infra::telemetry::init_telemetry();

    let builder = tauri::Builder::default();
    builder
        .register_uri_scheme_protocol("asset", move |ctx, request| {
            crate::delivery::protocols::asset::handler(ctx.app_handle(), &request)
        })
        .register_uri_scheme_protocol("video", move |ctx, request| {
            crate::delivery::protocols::video::handler(ctx.app_handle(), &request)
        })
        .register_uri_scheme_protocol("audio", move |ctx, request| {
            crate::delivery::protocols::audio::handler(ctx.app_handle(), &request)
        })
        .setup(|app| {
            // Resolve paths
            let app_data = app
                .path()
                .app_local_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_data).ok();

            let db_path = app_data.join("mundam.db");
            let settings_path = app_data.join("settings.json");
            let thumbnails_dir = app_data.join("thumbnails");
            std::fs::create_dir_all(&thumbnails_dir).ok();

            // Initialize Settings Infrastructure (Hexagonal)
            let settings_adapter = Arc::new(
                crate::infra::config::json_adapter::JsonSettingsAdapter::new(settings_path),
            );
            let settings_service =
                crate::feature::settings::SettingsService::new(settings_adapter.clone());
            app.manage(settings_service);

            // Initialize Event Bus (System Nervous System)
            let event_bus = Arc::new(TokioEventBus::new());
            app.manage(event_bus.clone() as Arc<dyn AppEventBus>);

            // Bridge Event Bus to Frontend
            {
                let mut rx = event_bus.subscribe();
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    while let Ok(event) = rx.recv().await {
                        // Standard Domain Event Bridge
                        if let Err(e) = app_handle.emit("mundam://domain-event", &event) {
                            tracing::error!("Failed to emit domain event to frontend: {}", e);
                        }

                        // Special Mapping for Legacy Indexer Events (Frontend compatibility)
                        match event {
                            crate::core::events::DomainEvent::ScanProgress { total, processed, current_file } => {
                                let _ = app_handle.emit("indexer:progress", serde_json::json!({
                                    "total": total,
                                    "processed": processed,
                                    "current_file": current_file,
                                }));
                            }
                            crate::core::events::DomainEvent::ScanCompleted { .. } => {
                                let _ = app_handle.emit("indexer:complete", 0);
                            }
                            _ => {}
                        }
                    }
                });
            }

            // Create the lifecycle registry — central hub for managing all background tasks
            let lifecycle = std::sync::Arc::new(LifecycleRegistry::new());
            app.manage(lifecycle.clone());

            // Initialize Format Registry (O(1) Router)
            let format_registry =
                std::sync::Arc::new(crate::core::formats::build_format_registry());
            app.manage(format_registry.clone());

            // Initialize HLS On-the-Fly Streaming Manager
            let hls_manager = crate::feature::transcoding::hls_manager::HlsManager::new(&app_data);
            app.manage(hls_manager.clone());
            {
                let manager = hls_manager.clone();
                let hls_token = lifecycle.child_token();
                tauri::async_runtime::spawn(async move {
                    manager.start_cleanup_worker(hls_token, 90).await;
                });
            }

            // Initialize DB and Worker (Blocking setup to avoid state race conditions)
            let handle = app.handle().clone();
            let lifecycle_for_setup = lifecycle.clone();
            tauri::async_runtime::block_on(async move {
                // Initialize Database Infrastructure
                let db_manager =
                    match crate::infra::database::manager::DbManager::new(&db_path).await {
                        Ok(manager) => Arc::new(manager),
                        Err(err) => {
                            tracing::error!("Failed to initialize database manager: {}", err);
                            return;
                        }
                    };
                handle.manage(db_manager.clone());

                // Initialize Query Handler early (needed by streaming server and IPC)
                let asset_query_handler: Arc<dyn crate::core::repository::AssetQueryHandler> =
                    Arc::new(crate::infra::database::queries::SqliteAssetQueries::new(
                        db_manager.pool().clone(),
                        format_registry.clone(),
                    ));
                handle.manage(asset_query_handler.clone());

                // Initialize Asset Ledger (Real SQLx Adapter)
                let asset_ledger: Arc<dyn crate::core::ledger::port::TransactionalAssetLedger> =
                    Arc::new(crate::infra::database::ledger::SqliteAssetLedger::new(
                        db_manager.pool().clone(),
                        event_bus.clone(),
                    ));
                handle.manage(asset_ledger.clone());

                // Initialize High-level query/search services
                let asset_query_service = crate::feature::assets::queries::AssetQueryService::new(
                    asset_query_handler.clone(),
                );
                handle.manage(asset_query_service);

                let search_query_handler =
                    crate::feature::search::SearchQueryHandler::new(asset_query_handler.clone());
                handle.manage(search_query_handler);

                // Generate a session token for streaming server authentication
                let session_token = uuid::Uuid::new_v4().to_string();
                handle.manage(
                    crate::delivery::tauri::commands::queries::StreamingSessionToken(
                        session_token.clone(),
                    ),
                );

                // Initialize Transcode Cache
                let transcode_cache = Arc::new(TranscodeCache::new(&app_data, format_registry.clone()));
                handle.manage(transcode_cache);

                // Start Streaming Server (Axum)
                let server_token = lifecycle_for_setup.child_token();
                let server_handle = start_server(handle.clone(), 9876, server_token.clone()).await;
                lifecycle_for_setup.register(
                    "streaming_server".to_string(),
                    server_token,
                    server_handle,
                );


                let priority_state = std::sync::Arc::new(
                    crate::core::workflows::thumbnails::priority::ThumbnailPriorityState::default(),
                );
                handle.manage(priority_state.clone());

                // Start Thumbnail Worker (Hybrid Queue)
                let thumbnail_token = lifecycle_for_setup.child_token();
                let thumbnail_worker =
                    crate::processing::workers::thumbnail_worker::ThumbnailWorker::new(
                        format_registry.clone(),
                        asset_ledger.clone(),
                        asset_query_handler.clone(),
                        priority_state,
                        thumbnails_dir.clone(),
                    );
                let thumbnail_handle = thumbnail_worker.start(thumbnail_token.clone());
                lifecycle_for_setup.register(
                    "thumbnail_worker".to_string(),
                    thumbnail_token,
                    thumbnail_handle,
                );

                // Start Color Worker (Reactive to Thumbnails)
                let color_worker = crate::processing::workers::color_worker::ColorWorker::new(
                    asset_ledger.clone(),
                    event_bus.clone(),
                    thumbnails_dir.to_path_buf(),
                );
                color_worker.start();

                // Start Watchers for Existing Roots
                if let Ok(roots) = asset_query_handler.list_folders(None).await {
                    tracing::info!("Starting watchers for {} roots", roots.len());

                    // Initialize Indexer
                    let indexer = Arc::new(crate::feature::library::indexer::LibraryIndexer::new(
                        asset_query_handler.clone(),
                        asset_ledger.clone(),
                        event_bus.clone(),
                        format_registry.clone(),
                    ));
                    handle.manage(indexer.clone());

                    indexer
                        .clone()
                        .start_event_listener(event_bus.clone())
                        .await;

                    // Initialize Watcher Service
                    let watcher = Arc::new(crate::processing::watcher::WatcherService::new(
                        event_bus.clone(),
                    ));
                    handle.manage(watcher.clone());

                    for folder in roots {
                        let root_path = std::path::PathBuf::from(&folder.path);

                        let watcher_token = lifecycle_for_setup.child_token();
                        if let Err(e) = watcher
                            .watch(root_path.clone(), watcher_token.clone())
                            .await
                        {
                            tracing::error!(
                                "Failed to start watcher for {}: {}",
                                root_path.display(),
                                e
                            );
                        } else {
                            lifecycle_for_setup.register(
                                format!("watcher:{}", root_path.display()),
                                watcher_token,
                                tauri::async_runtime::spawn(async {}), // Fake join handle as the watcher is self-managed
                            );
                        }
                    }
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
            // IPC Commands
            delivery::tauri::commands::queries::get_assets,
            delivery::tauri::commands::queries::get_asset,
            delivery::tauri::commands::queries::list_folders,
            delivery::tauri::commands::queries::list_tags,
            delivery::tauri::commands::queries::search_assets,
            delivery::tauri::commands::queries::get_tags_for_asset,
            delivery::tauri::commands::queries::get_all_subfolders,
            delivery::tauri::commands::queries::get_subfolder_counts,
            delivery::tauri::commands::queries::get_location_root_counts,
            delivery::tauri::commands::queries::get_smart_folders,
            delivery::tauri::commands::queries::get_library_stats,
            delivery::tauri::commands::queries::get_asset_exif,
            delivery::tauri::commands::queries::get_asset_colors,
            delivery::tauri::commands::queries::get_library_cache_stats,
            delivery::tauri::commands::mutations::create_folder,
            delivery::tauri::commands::mutations::remove_location,
            delivery::tauri::commands::mutations::start_indexing,
            delivery::tauri::commands::mutations::set_asset_folder,
            delivery::tauri::commands::mutations::update_asset_tags,
            delivery::tauri::commands::mutations::create_tag,
            delivery::tauri::commands::mutations::update_tag,
            delivery::tauri::commands::mutations::delete_tag,
            delivery::tauri::commands::mutations::add_tags_to_assets_batch,
            delivery::tauri::commands::mutations::remove_tags_from_assets_batch,
            delivery::tauri::commands::mutations::replace_tags_for_assets_batch,
            delivery::tauri::commands::mutations::save_smart_folder,
            delivery::tauri::commands::mutations::update_smart_folder,
            delivery::tauri::commands::mutations::delete_smart_folder,
            delivery::tauri::commands::mutations::update_asset_rating,
            delivery::tauri::commands::mutations::update_asset_notes,
            delivery::tauri::commands::mutations::reextract_asset_colors,
            delivery::tauri::commands::mutations::request_thumbnail_regenerate,
            delivery::tauri::commands::mutations::run_db_maintenance,
            delivery::tauri::commands::mutations::send_telemetry_log,
            delivery::tauri::commands::mutations::cleanup_cache,
            delivery::tauri::commands::mutations::clear_cache,
            delivery::tauri::commands::queries::get_library_supported_formats,
            delivery::tauri::commands::queries::get_audio_waveform_data,
            delivery::tauri::thumbnails::set_thumbnail_priority,
            // Settings Commands
            delivery::tauri::commands::settings::get_app_settings,
            delivery::tauri::commands::settings::update_app_settings,
            delivery::tauri::commands::settings::get_setting,
            delivery::tauri::commands::settings::set_setting,
            delivery::tauri::commands::queries::get_streaming_token,
            delivery::tauri::commands::streaming::needs_transcoding,
            delivery::tauri::commands::streaming::is_native_format,
            delivery::tauri::commands::streaming::get_stream_url,
            delivery::tauri::commands::streaming::get_quality_options,
            delivery::tauri::commands::streaming::ffmpeg_available,
            delivery::tauri::commands::streaming::is_cached,
            delivery::tauri::commands::streaming::get_streaming_cache_stats,
            delivery::tauri::commands::streaming::transcode_file,
            delivery::tauri::commands::streaming::cleanup_cache_streaming,
            delivery::tauri::commands::streaming::clear_cache_streaming
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                tracing::info!("Close requested. Orchestrating graceful shutdown.");
                api.prevent_close();

                let handle = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(lifecycle) = handle.try_state::<std::sync::Arc<LifecycleRegistry>>()
                    {
                        lifecycle.shutdown_all().await;
                    }
                    handle.exit(0);
                });
            }
        });
}
