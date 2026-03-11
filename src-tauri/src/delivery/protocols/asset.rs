use std::path::Path;
use std::sync::Arc;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

use crate::core::repository::AssetQueryHandler;
use crate::feature::transcoding::hls_manager::HlsManager;
use percent_encoding::percent_decode_str;

/// Setup the custom asset:// protocol for V2 Hexagonal Architecture
///
/// This protocol handler serves local physical media over HTTP, using
/// asynchronous `tokio::fs` calls to stream data chunks and support
/// HTTP 206 Partial Content queries. This prevents OOM errors when opening
/// massive files (e.g. 12GB MKV).
///
/// The handler supports the following URI formats:
/// - `asset://localhost/{asset_id}`: Serves the original file.
/// - `asset://localhost/{asset_id}?type=thumb`: Serves the thumbnail.
///
/// # Arguments
/// * `app_handle`: The Tauri application handle.
/// * `request`: The HTTP request.
///
/// # Returns
/// A `Response<Vec<u8>>` containing the file data.
///
/// # Examples
/// ```no_run
/// use tauri::http::{Request, Response, StatusCode};
/// use tauri::{AppHandle, Manager};
/// use crate::delivery::protocols::asset::handler;
///
/// let mut request = Request::builder()
///     .uri("asset://localhost/12345")
///     .body(Vec::new())
///     .unwrap();
///
/// let response = handler(&app_handle, &request);
/// assert_eq!(response.status(), StatusCode::OK);
/// ```
pub fn handler<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    request: &Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri().to_string();

    // Route to HLS Stream handler if this is a video chunk or playlist playback
    if uri.contains("/stream/hls/") {
        return hls_stream_handler(app_handle, request, &uri);
    }

    let (asset_id, is_thumb) = parse_asset_uri(&uri);

    // 1. Fetch the Repository (AssetQueryHandler) from the DI state
    let query_handler = match app_handle.try_state::<Arc<dyn AssetQueryHandler>>() {
        Some(handler) => handler,
        None => {
            tracing::error!("AssetQueryHandler state not found in Tauri AppHandle");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                b"Query handler not found".to_vec(),
            );
        }
    };

    // 2. Fetch the Asset using a blocking bridge since Tauri protocol is sync
    let asset_result =
        tauri::async_runtime::block_on(async { query_handler.get_by_id(&asset_id).await });

    let asset = match asset_result {
        Ok(Some(a)) => a,
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                b"Asset not found in database".to_vec(),
            )
        }
        Err(e) => {
            tracing::error!("Database error fetching asset {}: {:?}", asset_id, e);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                b"Database error".to_vec(),
            );
        }
    };

    // 3. Resolve physical path based on type
    let mut physical_path = asset.path;

    if is_thumb {
        let app_data = match app_handle.path().app_local_data_dir() {
            Ok(dir) => dir,
            Err(_) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    b"Data configuration path not found".to_vec(),
                )
            }
        };
        physical_path = app_data
            .join("thumbnails")
            .join(format!("{}.webp", asset.id));
    }

    if !physical_path.exists() {
        return error_response(
            StatusCode::NOT_FOUND,
            b"File not found on local disk".to_vec(),
        );
    }

    // 4. Delegate to the async chunking I/O logic
    let range_header = request.headers().get(header::RANGE).cloned();

    // As the current URI scheme handler for Tauri is synchronous, we block on Tokio tasks here
    let response_result = tauri::async_runtime::block_on(async {
        serve_file_async(&physical_path, range_header.as_ref()).await
    });

    match response_result {
        Ok(res) => res,
        Err(res) => res, // The Err type is also an identical HTTP error Response struct
    }
}

/// Helper to wrap the error response creation
///
/// # Arguments
/// * `status`: The HTTP status code.
/// * `body`: The response body.
///
/// # Returns
/// A `Response<Vec<u8>>` containing the error data.
///
/// # Examples
/// ```no_run
/// use tauri::http::{Response, StatusCode};
/// use crate::delivery::protocols::asset::error_response;
///
/// let response = error_response(StatusCode::NOT_FOUND, b"Asset not found".to_vec());
/// assert_eq!(response.status(), StatusCode::NOT_FOUND);
/// ```
fn error_response(status: StatusCode, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body)
        .unwrap_or_else(|_| Response::default())
}

/// Helper to decode URI, and test if `type=thumb` is present.
///
/// # Arguments
/// * `uri`: The URI to parse.
///
/// # Returns
/// A tuple containing the asset ID and a boolean indicating if it's a thumbnail.
///
/// # Examples
/// ```no_run
/// use crate::delivery::protocols::asset::parse_asset_uri;
///
/// let (asset_id, is_thumb) = parse_asset_uri("asset://localhost/12345?type=thumb");
/// assert_eq!(asset_id, "12345");
/// assert!(is_thumb);
/// ```
fn parse_asset_uri(uri: &str) -> (String, bool) {
    let prefix = "asset://localhost/";
    let fallback = "asset://";

    let path_with_query = if uri.starts_with(prefix) {
        &uri[prefix.len()..]
    } else if uri.starts_with(fallback) {
        &uri[fallback.len()..]
    } else {
        uri
    };

    let (path_part, query_part) = if let Some(pos) = path_with_query.find('?') {
        (&path_with_query[..pos], Some(&path_with_query[pos + 1..]))
    } else {
        (path_with_query, None)
    };

    let decoded_id = percent_decode_str(path_part)
        .decode_utf8_lossy()
        .into_owned();
    let is_thumb = query_part.is_some_and(|q| q.contains("type=thumb"));

    (decoded_id, is_thumb)
}

/// Async tokio file streaming and HTTP Range chunking mechanism
///
/// # Arguments
/// * `path`: The path to the file to serve.
/// * `range`: The range header to use for partial content.
///
/// # Returns
/// A `Result<Response<Vec<u8>>, Response<Vec<u8>>>` containing the file data.
///
/// # Examples
/// ```no_run
/// use tauri::http::{Response, StatusCode};
/// use crate::delivery::protocols::asset::serve_file_async;
///
/// let response = serve_file_async(&PathBuf::from("test.mkv"), None);
/// assert_eq!(response.status(), StatusCode::OK);
/// ```
async fn serve_file_async(
    path: &Path,
    range: Option<&tauri::http::HeaderValue>,
) -> Result<Response<Vec<u8>>, Response<Vec<u8>>> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    let mut file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(e) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string().into_bytes(),
            ))
        }
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(_) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                b"Cannot read file metadata".to_vec(),
            ))
        }
    };

    let file_size = metadata.len();

    // Simplistic MIME detection. Use fallback octet setup if unfound.
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let builder = Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            "Content-Range, Content-Length, Accept-Ranges",
        );

    // Partial Content Read (Chunking limit active)
    if let Some(range_value) = range {
        if let Ok(range_str) = range_value.to_str() {
            if let Some(range_spec) = range_str.strip_prefix("bytes=") {
                let mut start: u64 = 0;
                let mut end: u64 = file_size.saturating_sub(1);

                let parts: Vec<&str> = range_spec.split('-').collect();
                if !parts.is_empty() && !parts[0].is_empty() {
                    if let Ok(s) = parts[0].parse::<u64>() {
                        start = s;
                    }
                }

                if parts.len() >= 2 && !parts[1].is_empty() {
                    if let Ok(e) = parts[1].parse::<u64>() {
                        end = e;
                    }
                } else if parts.len() == 1 && range_spec.starts_with('-') {
                    if let Ok(suffix) = range_spec[1..].parse::<u64>() {
                        start = file_size.saturating_sub(suffix);
                        end = file_size.saturating_sub(1);
                    }
                }

                if start >= file_size {
                    return Err(error_response(
                        StatusCode::RANGE_NOT_SATISFIABLE,
                        format!("bytes */{}", file_size).into_bytes(),
                    ));
                }
                if end >= file_size {
                    end = file_size.saturating_sub(1);
                }
                if start > end {
                    end = start;
                }

                // Chunk Size Rule: Max limit bounded (10 MB chunks per UI Network Request).
                let max_chunk = 10 * 1024 * 1024;
                let requested_size = (end - start) + 1;
                let chunk_size = std::cmp::min(requested_size, max_chunk);

                let mut buffer = vec![0u8; chunk_size as usize];
                if file.seek(std::io::SeekFrom::Start(start)).await.is_ok() {
                    if let Ok(bytes_read) = file.read_exact(&mut buffer).await {
                        // Truncate in case read_exact returned fewer somehow
                        buffer.truncate(bytes_read);
                        return Ok(builder
                            .status(StatusCode::PARTIAL_CONTENT)
                            .header(
                                header::CONTENT_RANGE,
                                format!(
                                    "bytes {}-{}/{}",
                                    start,
                                    start + bytes_read as u64 - 1,
                                    file_size
                                ),
                            )
                            .header(header::CONTENT_LENGTH, bytes_read as u64)
                            .body(buffer)
                            .unwrap_or_else(|_| Response::default()));
                    }
                }
            }
        }
    }

    // Default Request Strategy: No specific byte boundaries requested
    // Note: If media is immense, prevent the frontend process from fetching in 1 blow.
    if file_size > 500 * 1024 * 1024 {
        let chunk_size = std::cmp::min(file_size, 10 * 1024 * 1024);
        let mut buffer = vec![0u8; chunk_size as usize];
        if file.seek(std::io::SeekFrom::Start(0)).await.is_ok() {
            if let Ok(bytes_read) = file.read_exact(&mut buffer).await {
                buffer.truncate(bytes_read);
                return Ok(builder
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes 0-{}/{}", bytes_read as u64 - 1, file_size),
                    )
                    .header(header::CONTENT_LENGTH, bytes_read as u64)
                    .body(buffer)
                    .unwrap_or_else(|_| Response::default()));
            }
        }
    }

    // For standard small files simply read whole file
    let mut all_data = Vec::with_capacity(file_size as usize);
    if file.read_to_end(&mut all_data).await.is_err() {
        return Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            b"Read disk error".to_vec(),
        ));
    }

    Ok(builder
        .status(StatusCode::OK)
        .header(header::CONTENT_LENGTH, file_size)
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(all_data)
        .unwrap_or_else(|_| Response::default()))
}

/// Specialized subset handler for On-the-Fly HLS Delivery
///
/// Converts a specific physical asset chunk to an HLS format by wrapping `HlsManager`.
///
/// # Arguments
/// * `app_handle` - Injected app instance
/// * `request` - The HTTP Request
/// * `uri` - Extracted raw URI string
fn hls_stream_handler<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    request: &Request<Vec<u8>>,
    uri: &str,
) -> Response<Vec<u8>> {
    // Expected URI: asset://localhost/stream/hls/{asset_id}/playlist.m3u8
    // or asset://localhost/stream/hls/{asset_id}/segment_00001.ts
    // Extract asset_id and the target file name
    let prefix = "asset://localhost/stream/hls/";
    let path_with_query = if uri.starts_with(prefix) {
        &uri[prefix.len()..]
    } else {
        return error_response(StatusCode::BAD_REQUEST, b"Invalid HLS URI".to_vec());
    };

    // Split on queries ignoring them if any
    let (path_part, _query_part) = if let Some(pos) = path_with_query.find('?') {
        (&path_with_query[..pos], Some(&path_with_query[pos + 1..]))
    } else {
        (path_with_query, None)
    };

    // Split into asset_id and filename
    let parts: Vec<&str> = path_part.split('/').collect();
    if parts.len() < 2 {
        return error_response(StatusCode::BAD_REQUEST, b"Missing asset_id or filename".to_vec());
    }

    let asset_id = parts[0];
    let file_name = parts[1];

    let hls_manager = match app_handle.try_state::<Arc<HlsManager>>() {
        Some(manager) => manager,
        None => {
            tracing::error!("HlsManager not found in Tauri AppState.");
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, b"HLS Manager state missing".to_vec());
        }
    };

    if file_name == "playlist.m3u8" {
        // Find the physical path from DB first
        let query_handler = match app_handle.try_state::<Arc<dyn AssetQueryHandler>>() {
            Some(handler) => handler,
            None => {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, b"Query handler not found".to_vec());
            }
        };

        let asset_result = tauri::async_runtime::block_on(async { query_handler.get_by_id(asset_id).await });
        let asset = match asset_result {
            Ok(Some(a)) => a,
            Ok(None) => return error_response(StatusCode::NOT_FOUND, b"Asset not found".to_vec()),
            Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, b"Database error".to_vec()),
        };

        let original_path = asset.path;
        if !original_path.exists() {
            return error_response(StatusCode::NOT_FOUND, b"Original media missing on disk".to_vec());
        }

        let mime = mime_guess::from_path(&original_path).first_or_octet_stream().to_string();

        let session_dir_res = tauri::async_runtime::block_on(async {
            hls_manager.get_or_start_stream(asset_id, &original_path, Some(&mime)).await
        });

        let session_dir = match session_dir_res {
            Ok(dir) => dir,
            Err(e) => {
                tracing::error!("HLS spawn failed: {:?}", e);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, b"HLS transcode pipeline failed".to_vec());
            }
        };

        let playlist_path = session_dir.join("playlist.m3u8");

        // Wait up to ~10 seconds for the first playlist chunks to be available parsing by FFmpeg
        let mut tries = 0;
        while !playlist_path.exists() && tries < 50 {
            std::thread::sleep(std::time::Duration::from_millis(200));
            tries += 1;
        }

        if !playlist_path.exists() {
            return error_response(StatusCode::GATEWAY_TIMEOUT, b"Playlist segmenting timed out".to_vec());
        }

        let content = std::fs::read(&playlist_path).unwrap_or_default();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/vnd.apple.mpegurl")
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(content)
            .unwrap_or_else(|_| Response::default());

    } else if file_name.ends_with(".ts") {
        hls_manager.touch_session(asset_id);

        let session_dir = hls_manager.streams_dir.join(asset_id);
        let segment_path = session_dir.join(file_name);

        let range_header = request.headers().get(header::RANGE).cloned();
        let response_result = tauri::async_runtime::block_on(async {
            serve_file_async(&segment_path, range_header.as_ref()).await
        });

        return match response_result {
            Ok(mut res) => {
                // Ensure proper content type for segments
                res.headers_mut().insert(header::CONTENT_TYPE, "video/MP2T".parse().unwrap());
                // Short cache control for segments
                res.headers_mut().insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
                res
            },
            Err(res) => res, // 404 or Read errors naturally propagated
        };
    }

    error_response(StatusCode::BAD_REQUEST, b"Only .m3u8 and .ts are allowed for HLS pipeline".to_vec())
}
