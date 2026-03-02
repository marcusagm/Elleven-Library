#![allow(clippy::unwrap_used)]
//! Axum HTTP Server for HLS Streaming
//!
//! Runs on a separate thread and provides endpoints for:
//! - /health - Health check
//! - /probe/{path} - Get video metadata and native format detection
//! - /playlist/{path} - Generate M3U8 playlist dynamically
//! - /segment/{path}/{index} - Transcode and serve video segments
//!
//! # Security
//!
//! Three layers of defense-in-depth:
//! 1. **CORS Restriction** — Only Tauri webview origins are allowed.
//! 2. **Session Token** — A UUID v4 token generated at boot, required on every request.
//! 3. **Path Scope Validation** — Only files within user-authorized library folders are served.

use axum::extract::Query;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::async_runtime::JoinHandle;
use tauri::Manager;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;

use super::{
    helpers::StreamError, linear::LinearManager, playlist, probe, process_manager::ProcessManager,
    segment,
};
use crate::db::Db;
use crate::error::AppError;
use crate::transcoding::cache::TranscodeCache;
use tracing::{error, info, instrument};

/// Default port for the HLS streaming server
pub const DEFAULT_PORT: u16 = 9876;

/// Segment duration in seconds
pub const SEGMENT_DURATION: f64 = 10.0;

/// Shared state for the streaming server
#[derive(Clone)]
pub struct AppState {
    /// Shared transcoding cache.
    pub cache: Arc<TranscodeCache>,
    /// Shared process manager for tracking ffmpeg processes.
    pub process_manager: Arc<RwLock<ProcessManager>>,
    /// Manager for linear HLS streaming sessions.
    pub linear_manager: LinearManager,
    /// Handle to the Tauri application.
    pub app_handle: tauri::AppHandle,
    /// Database handle for path scope validation.
    pub database: Arc<Db>,
    /// Session token for request authentication.
    pub session_token: String,
}

/// The HLS Streaming Server
pub struct StreamingServer {
    /// Port to bind the server on.
    port: u16,
    /// Handle to the Tauri application.
    app_handle: tauri::AppHandle,
    /// Database handle for path scope validation.
    database: Arc<Db>,
    /// Session token for request authentication.
    session_token: String,
}

impl StreamingServer {
    /// Create a new streaming server instance
    pub fn new(
        port: u16,
        app_handle: tauri::AppHandle,
        database: Arc<Db>,
        session_token: String,
    ) -> Self {
        Self {
            port,
            app_handle,
            database,
            session_token,
        }
    }

    /// Start the server on a background task.
    ///
    /// Accepts a `CancellationToken` for graceful shutdown. When the token is
    /// cancelled, the axum server stops accepting new connections, in-flight
    /// requests are completed, and internal cleanup loops exit cooperatively.
    pub async fn start(
        self,
        token: CancellationToken,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app_data = self
            .app_handle
            .path()
            .app_local_data_dir()
            .map_err(|dir_error| format!("Failed to get app data dir: {}", dir_error))?;

        let cache = Arc::new(TranscodeCache::new(&app_data));
        let process_manager = Arc::new(RwLock::new(ProcessManager::new()));
        let linear_manager = LinearManager::new(self.app_handle.clone());

        let state = AppState {
            cache,
            process_manager: process_manager.clone(),
            linear_manager: linear_manager.clone(),
            app_handle: self.app_handle.clone(),
            database: self.database.clone(),
            session_token: self.session_token.clone(),
        };

        // Spawn cleanup task for stale processes (child token for cancellation)
        let process_cleanup_token = token.child_token();
        let process_manager_clone = process_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = process_cleanup_token.cancelled() => {
                        info!("Process cleanup task shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        let mut process_manager_guard = process_manager_clone.write().await;
                        process_manager_guard.cleanup_stale(30); // 30 seconds timeout
                    }
                }
            }
        });

        // Spawn cleanup task for linear sessions (child token for cancellation)
        let linear_cleanup_token = token.child_token();
        let linear_manager_clone = linear_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                tokio::select! {
                    _ = linear_cleanup_token.cancelled() => {
                        info!("Linear session cleanup task shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        // Cleanup sessions inactive for 60 seconds
                        linear_manager_clone.cleanup(Duration::from_secs(60)).await;
                    }
                }
            }
        });

        let cors = build_cors_layer();

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/probe/*path", get(probe_handler))
            .route("/playlist/*path", get(playlist_handler))
            .route("/segment/*path", get(segment_handler))
            // New routes for linear HLS
            .route("/hls-live/*path", get(linear_hls_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                validate_session_token,
            ))
            .layer(cors)
            .with_state(state);

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("HLS streaming server started on http://{}", addr);

        // Graceful shutdown: axum stops accepting new connections when the token is cancelled,
        // but finishes processing in-flight requests before returning.
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                token.cancelled().await;
                info!("Streaming server received shutdown signal");
            })
            .await?;

        info!("Streaming server stopped");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Security: CORS, Token Validation, Path Scope
// ---------------------------------------------------------------------------

/// Build a restricted CORS layer that only allows Tauri webview origins.
///
/// In production, Tauri uses `tauri://localhost` (macOS/Linux) or
/// `https://tauri.localhost` (Windows). During development, the Vite dev
/// server runs on `http://localhost:1420`.
fn build_cors_layer() -> CorsLayer {
    let allowed_origins = [
        // Tauri production origins
        "tauri://localhost",
        "https://tauri.localhost",
        // Vite dev server
        "http://localhost:1420",
    ];

    let parsed_origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(parsed_origins)
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::RANGE,
            header::ACCEPT,
            header::ORIGIN,
        ])
}

/// Middleware that validates the session token on every request except `/health`.
///
/// The token must be provided as a `?token=xxx` query parameter.
/// Returns 401 Unauthorized if the token is missing or invalid.
async fn validate_session_token(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // Allow health checks without authentication
    if request.uri().path() == "/health" {
        return next.run(request).await;
    }

    let query_string = request.uri().query().unwrap_or("");

    // Simple query parameter extraction without external crate dependency.
    // We only need the "token" parameter, so a basic split-based approach suffices.
    let provided_token = query_string.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some("token"), Some(value)) => Some(value),
            _ => None,
        }
    });

    match provided_token {
        Some(token_value) if token_value == state.session_token => next.run(request).await,
        _ => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid or missing session token"))
            .unwrap_or_default(),
    }
}

/// Validates that a file path is within one of the user's authorized library folders.
///
/// Uses `canonicalize()` to resolve symlinks and `..` traversal, then checks
/// that the resolved path starts with at least one registered root folder.
///
/// # Errors
///
/// Returns an error message if the path is outside the authorized scope
/// or if canonicalization fails.
async fn validate_path_scope(database: &Db, file_path: &std::path::Path) -> Result<(), String> {
    let path_to_check = file_path.to_path_buf();

    // Canonicalize in a blocking context since it performs filesystem I/O
    let canonical_path = tokio::task::spawn_blocking(move || path_to_check.canonicalize())
        .await
        .map_err(|join_error| format!("Path validation failed: {}", join_error))?
        .map_err(|io_error| format!("Cannot resolve path: {}", io_error))?;

    let root_folders = database
        .get_all_root_folders()
        .await
        .map_err(|db_error| format!("Failed to query authorized folders: {}", db_error))?;

    let is_within_authorized_scope = root_folders.iter().any(|(_id, root_path)| {
        let root = std::path::Path::new(root_path);
        canonical_path.starts_with(root)
    });

    if is_within_authorized_scope {
        Ok(())
    } else {
        Err(format!(
            "Access denied: path {:?} is outside authorized library folders",
            file_path
        ))
    }
}

/// Helper that builds a 403 Forbidden response for path scope violations.
fn forbidden_response(message: String) -> StreamError {
    StreamError(AppError::Generic(message))
}

// ---------------------------------------------------------------------------
// Route Handlers
// ---------------------------------------------------------------------------

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Probe endpoint - returns video metadata
#[instrument(skip(state))]
async fn probe_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, StreamError> {
    let file_path = decode_path(&path);

    // Validate path is within authorized library folders
    validate_path_scope(&state.database, &file_path)
        .await
        .map_err(|e| forbidden_response(e))?;

    match probe::get_video_info(&state.app_handle, &file_path).await {
        Ok(info) => {
            let json = serde_json::to_string(&info).unwrap_or_default();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json))
                .unwrap_or_default())
        }
        Err(probe_error) => {
            error!(path = ?file_path, "Probe failed: {}", probe_error);
            Err(StreamError(AppError::Generic(format!(
                "Probe failed: {}",
                probe_error
            ))))
        }
    }
}

/// Playlist endpoint - generates M3U8 dynamically
#[instrument(skip(state))]
async fn playlist_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StreamError> {
    let file_path = decode_path(&path);

    // Validate path is within authorized library folders
    validate_path_scope(&state.database, &file_path)
        .await
        .map_err(|e| forbidden_response(e))?;

    let quality = params
        .get("quality")
        .map(|s| s.as_str())
        .unwrap_or("standard");

    // Preserve the token in segment URLs so HLS.js can authenticate each segment request
    let token_param = params
        .get("token")
        .map(|token| format!("&token={}", token))
        .unwrap_or_default();

    // First, probe the video to get duration
    let info = match probe::get_video_info(&state.app_handle, &file_path).await {
        Ok(video_info) => video_info,
        Err(probe_error) => {
            return Err(StreamError(AppError::Generic(format!(
                "Failed to probe video: {}",
                probe_error
            ))));
        }
    };

    let m3u8 = playlist::generate_m3u8_with_token(
        &path,
        info.duration_secs,
        SEGMENT_DURATION,
        quality,
        &token_param,
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(m3u8))
        .unwrap_or_default())
}

/// Segment endpoint - transcodes and serves a video segment
#[instrument(skip(state))]
async fn segment_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StreamError> {
    let quality = params
        .get("quality")
        .map(|s| s.as_str())
        .unwrap_or("standard");
    // Path format: /segment/{encoded_file_path}/{index}
    // We need to parse out the index from the end
    let (file_path, index) = match parse_segment_path(&path) {
        Some((parsed_path, parsed_index)) => (parsed_path, parsed_index),
        None => {
            return Err(StreamError(AppError::Generic(
                "Invalid segment path format".to_string(),
            )));
        }
    };

    // Validate path is within authorized library folders
    validate_path_scope(&state.database, &file_path)
        .await
        .map_err(|e| forbidden_response(e))?;

    match segment::get_segment(
        &state.app_handle,
        &state.cache,
        &state.process_manager,
        &file_path,
        index,
        SEGMENT_DURATION,
        quality,
    )
    .await
    {
        Ok(data) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "video/MP2T")
            .header(header::CACHE_CONTROL, "max-age=3600")
            .body(Body::from(data))
            .unwrap_or_default()),
        Err(segment_error) => {
            error!("SEGMENT_ERROR: {}", segment_error);
            Err(StreamError(AppError::Generic(format!(
                "Segment failed: {}",
                segment_error
            ))))
        }
    }
}

/// Linear HLS Handler using /hls-live/*path
/// Request can be:
/// 1. .../video.swf/index.m3u8 -> Starts transcode, returns playlist
/// 2. .../video.swf/segment_00001.ts -> Returns segment
#[instrument(skip(state))]
async fn linear_hls_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StreamError> {
    // 1. Handle Playlist Request
    if path.ends_with("/index.m3u8") {
        // Strip the suffix
        let raw_path = path.trim_end_matches("/index.m3u8");
        // Decode the file path
        let decoded_path = urlencoding::decode(raw_path)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| raw_path.to_string());

        let file_path = PathBuf::from(decoded_path);

        // Validate path is within authorized library folders
        validate_path_scope(&state.database, &file_path)
            .await
            .map_err(|e| forbidden_response(e))?;

        if !file_path.exists() {
            return Err(StreamError(AppError::NotFound(format!(
                "File not found: {:?}",
                file_path
            ))));
        }

        let quality = params
            .get("quality")
            .map(|s| s.as_str())
            .unwrap_or("standard");

        match state.linear_manager.get_or_start(&file_path, quality).await {
            Ok(temp_dir) => {
                let playlist_path = temp_dir.join("index.m3u8");

                // Poll for playlist existence (ffmpeg might take a second to create it)
                let mut tries = 0;
                // Increased timeout to 15s to allow for slower ffmpeg startup
                while !playlist_path.exists() && tries < 150 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    tries += 1;
                }

                if playlist_path.exists() {
                    match tokio::fs::read_to_string(&playlist_path).await {
                        Ok(content) => {
                            // Inject the session token into all segment URLs
                            let tokenized_content = content
                                .lines()
                                .map(|line| {
                                    if line.ends_with(".ts") {
                                        format!("{}?token={}\n", line, state.session_token)
                                    } else {
                                        format!("{}\n", line)
                                    }
                                })
                                .collect::<String>();

                            Ok(Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
                                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                                .body(Body::from(tokenized_content))
                                .unwrap_or_default())
                        }
                        Err(read_error) => Err(StreamError(AppError::Generic(format!(
                            "Failed to read playlist: {}",
                            read_error
                        )))),
                    }
                } else {
                    Err(StreamError(AppError::Generic(
                        "Playlist generation timed out".into(),
                    )))
                }
            }
            Err(start_error) => Err(StreamError(AppError::Generic(format!(
                "Failed to start streaming: {}",
                start_error
            )))),
        }
    }
    // 2. Handle Segment Request
    else if path.ends_with(".ts") {
        // Path format: /encoded/path/to/file.ext/segment_00001.ts
        if let Some(last_slash) = path.rfind('/') {
            let file_part_raw = &path[..last_slash];
            let segment_name = &path[last_slash + 1..];

            // Decode file path
            let decoded_file_path = urlencoding::decode(file_part_raw)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| file_part_raw.to_string());

            let file_path = PathBuf::from(decoded_file_path);

            // Validate path is within authorized library folders
            validate_path_scope(&state.database, &file_path)
                .await
                .map_err(|e| forbidden_response(e))?;

            if let Some(temp_dir) = state.linear_manager.get_temp_dir(&file_path).await {
                let segment_path = temp_dir.join(segment_name);
                if segment_path.exists() {
                    match tokio::fs::read(&segment_path).await {
                        Ok(data) => Ok(Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "video/MP2T")
                            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                            .header(header::CACHE_CONTROL, "no-cache")
                            .body(Body::from(data))
                            .unwrap_or_default()),
                        Err(read_error) => {
                            error!("Error reading segment {:?}: {}", segment_path, read_error);
                            Err(StreamError(AppError::NotFound(
                                "Segment read failed".into(),
                            )))
                        }
                    }
                } else {
                    Err(StreamError(AppError::NotFound(
                        "Segment file not found".into(),
                    )))
                }
            } else {
                Err(StreamError(AppError::NotFound(
                    "Session not active for this file".into(),
                )))
            }
        } else {
            Err(StreamError(AppError::Generic(
                "Invalid segment path".into(),
            )))
        }
    } else {
        Err(StreamError(AppError::Generic(
            "Invalid HLS Live request path".into(),
        )))
    }
}

/// Decode URL-encoded path
fn decode_path(path: &str) -> PathBuf {
    // URL decode the path first
    let decoded = urlencoding::decode(path)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| path.to_string());

    // The path comes as /path/to/file (the leading slash is part of the route)
    // For Unix absolute paths like /Users/..., we need to preserve them
    // For Windows paths like C:\..., they would be encoded differently

    // If path starts with "/" and next char is not "/", it's an absolute Unix path
    // that was passed directly, so preserve it
    if decoded.starts_with('/') {
        // Check if it looks like a Unix absolute path (e.g., /Users, /home, /tmp)
        let parts: Vec<&str> = decoded.splitn(3, '/').collect();
        if parts.len() >= 2 && !parts[1].is_empty() {
            // Valid Unix path like /Users/...
            return PathBuf::from(&decoded);
        }
    }

    // Fallback: treat as relative path
    PathBuf::from(decoded)
}

/// Parse segment path to extract file path and index
/// Format: {url_encoded_path}/{index}
fn parse_segment_path(path: &str) -> Option<(PathBuf, u32)> {
    // URL decode first
    let decoded = urlencoding::decode(path)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| path.to_string());

    // Find the last slash to separate index from path
    if let Some(last_slash) = decoded.rfind('/') {
        let file_part = &decoded[..last_slash];
        let index_part = &decoded[last_slash + 1..];

        // Try to parse index (might have .ts extension)
        let index_str = index_part.trim_end_matches(".ts");
        if let Ok(index) = index_str.parse::<u32>() {
            return Some((PathBuf::from(file_part), index));
        }
    }

    None
}

/// Start the streaming server in a background task.
///
/// Returns the `JoinHandle` so it can be registered in the `LifecycleRegistry`.
/// The server shuts down gracefully when the provided `CancellationToken` is cancelled.
///
/// # Arguments
///
/// * `app_handle` - Handle to the Tauri application.
/// * `token` - Cancellation token for graceful shutdown.
/// * `database` - Shared database handle for path scope validation.
/// * `session_token` - UUID v4 session token for request authentication.
pub fn spawn_server(
    app_handle: tauri::AppHandle,
    token: CancellationToken,
    database: Arc<Db>,
    session_token: String,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let server = StreamingServer::new(DEFAULT_PORT, app_handle, database, session_token);
        if let Err(server_error) = server.start(token).await {
            eprintln!("ERROR: HLS streaming server failed: {}", server_error);
        }
    })
}
