//! HTTP Streaming Server (Axum)
//!
//! A high-performance embedded HTTP server for media delivery.
//! Supported features:
//! - Range Requests (HTTP 206) for MP4/WebM.
//! - HLS Playlist (.m3u8) and Segment (.ts) serving.
//! - Session Token Authentication.
//! - Asset ID to Physical Path resolution via AssetQueryHandler.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::Path as StdPath;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;
use tokio::io::AsyncReadExt;

use crate::core::repository::AssetQueryHandler;
use crate::delivery::tauri::commands::queries::StreamingSessionToken;
use crate::feature::transcoding::hls_manager::HlsManager;
use crate::core::formats::FormatRegistry;

/// Shared server state for Axum handlers.
#[derive(Clone)]
struct ServerState {
    app_handle: AppHandle,
    asset_query: Arc<dyn AssetQueryHandler>,
    hls_manager: Arc<HlsManager>,
    format_registry: Arc<FormatRegistry>,
}

#[derive(Deserialize)]
struct StreamQuery {
    token: String,
}

/// Initializes and starts the Axum streaming server.
pub async fn start_server(app_handle: AppHandle, port: u16) -> tauri::async_runtime::JoinHandle<()> {
    let asset_query = app_handle
        .state::<Arc<dyn AssetQueryHandler>>()
        .inner()
        .clone();
    let hls_manager = app_handle.state::<Arc<HlsManager>>().inner().clone();
    let format_registry = app_handle.state::<Arc<FormatRegistry>>().inner().clone();

    let state = ServerState {
        app_handle,
        asset_query,
        hls_manager,
        format_registry,
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/probe/:asset_id", get(probe_handler))
        .route("/stream/:asset_id", get(stream_handler))
        .route("/playlist/:asset_id/playlist.m3u8", get(playlist_handler))
        .route("/segment/:asset_id/:segment", get(segment_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(CorsLayer::permissive()) // More restrictive CORS can be added if needed
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Streaming server listening on {}", addr);

    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(error) => {
                tracing::error!("Failed to bind streaming server to {}: {}", addr, error);
                return;
            }
        };
        if let Err(error) = axum::serve(listener, app).await {
            tracing::error!("Streaming server exited with error: {}", error);
        }
    })
}

/// Middleware to validate the session token.
async fn auth_middleware(
    State(state): State<ServerState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Health check doesn't need token
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let query: Query<StreamQuery> = Query::try_from_uri(req.uri()).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let session_token = state.app_handle.state::<StreamingSessionToken>();

    if query.token != session_token.0 {
        tracing::warn!("Unauthorized streaming request: invalid token");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

async fn health_handler() -> &'static str {
    "OK"
}

/// Handler for probing media metadata.
async fn probe_handler(
    State(state): State<ServerState>,
    Path(asset_id): Path<String>,
) -> Result<Response, StatusCode> {
    let asset = state
        .asset_query
        .get_by_id(&asset_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Detect format and extract metadata (V1 Parity)
    if let Some(format) = state.format_registry.detect(&asset.path) {
        if let Some(provider) = state.format_registry.get_provider(&format.name) {
            if let Some(metadata_cap) = provider.metadata() {
                let metadata: serde_json::Value = metadata_cap
                    .extract_technical(&asset.path)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to extract technical metadata: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;

                return Ok(Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .body(Body::from(serde_json::to_string(&metadata).unwrap_or_default()))
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
            }
        }
    }

    // Fallback or empty metadata if no specialized metadata capability exists
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(r#"{"streams":[], "format":{}}"#))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}

/// Handler for direct media streaming with Range support.
async fn stream_handler(
    headers: HeaderMap,
    State(state): State<ServerState>,
    Path(asset_id): Path<String>,
) -> Result<Response, StatusCode> {
    let asset = state
        .asset_query
        .get_by_id(&asset_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let range = headers.get(header::RANGE).cloned();
    serve_file(&asset.path, range).await
}

/// Handler for HLS playlists.
async fn playlist_handler(
    State(state): State<ServerState>,
    Path(asset_id): Path<String>,
) -> Result<Response, StatusCode> {
    let asset = state
        .asset_query
        .get_by_id(&asset_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let path = asset.path;
    let mime = mime_guess::from_path(&path).first_or_octet_stream().to_string();

    let session_dir = state
        .hls_manager
        .get_or_start_stream(&asset_id, &path, Some(&mime))
        .await
        .map_err(|e| {
            tracing::error!("Failed to start HLS stream: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let playlist_path = session_dir.join("playlist.m3u8");

    // Poll for playlist existence (FFmpeg takes a moment to write it)
    let mut tries = 0;
    while !playlist_path.exists() && tries < 25 {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        tries += 1;
    }

    if !playlist_path.exists() {
        return Err(StatusCode::GATEWAY_TIMEOUT);
    }

    let content = tokio::fs::read(&playlist_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Response::builder()
        .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handler for HLS segments.
async fn segment_handler(
    State(state): State<ServerState>,
    Path((asset_id, segment)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    state.hls_manager.touch_session(&asset_id);

    let segment_path = state.hls_manager.streams_dir.join(&asset_id).join(segment);
    if !segment_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    serve_file(&segment_path, None).await
}

/// Helper to serve a file using tokio and support Range headers.
async fn serve_file(path: &StdPath, range: Option<header::HeaderValue>) -> Result<Response, StatusCode> {
    let file = File::open(path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    let metadata = file.metadata().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let size = metadata.len();

    let mime = mime_guess::from_path(path).first_or_octet_stream().to_string();
    
    // Simple Range handling (can be optimized with tower_http::services::ServeFile if preferred)
    // For now mirroring the logic from V1/Asset protocol but in Axum style.
    
    if let Some(range_header) = range.and_then(|h| h.to_str().ok().map(|s| s.to_owned())) {
        if let Some(range_spec) = range_header.strip_prefix("bytes=") {
            let parts: Vec<&str> = range_spec.split('-').collect();
            let start = parts[0].parse::<u64>().unwrap_or(0);
            let end = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(size - 1);

            if start >= size {
                return Ok((StatusCode::RANGE_NOT_SATISFIABLE, format!("bytes */{}", size)).into_response());
            }

            let end = end.min(size - 1);
            let length = end - start + 1;

            let mut file = file;
            use tokio::io::AsyncSeekExt;
            file.seek(tokio::io::SeekFrom::Start(start)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            // Use explicit take from AsyncReadExt
            let stream = ReaderStream::new(AsyncReadExt::take(file, length));
            let body = Body::from_stream(stream);

            return Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, size))
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_LENGTH, length)
                .body(body)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, size)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
