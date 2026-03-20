use std::path::Path;
use std::sync::Arc;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::{AppHandle, Manager};

use crate::core::AppResult;
use crate::core::repository::AssetQueryHandler;
use crate::delivery::protocols::common::{error_response, serve_file};
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
/// - `asset://localhost/{asset_id}?type=preview`: Serves a high-resolution preview.
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

    let (asset_id, requested_type) = parse_asset_uri(&uri);
    let is_thumb = requested_type == Some("thumb".to_string());
    let is_preview = requested_type == Some("preview".to_string());
    let is_glb = requested_type == Some("glb".to_string());

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
    let mut physical_path = asset.path.clone();

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
    } else if is_glb {
        // Only use thumbnail cache if original is NOT already a GLB/GLTF
        let path_str = physical_path.to_string_lossy().to_lowercase();
        if !path_str.ends_with(".glb") && !path_str.ends_with(".gltf") {
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
                .join(format!("{}.glb", asset.id));
        }
    } else if is_preview {
        // High-Resolution Preview Logic (V1 Parity)
        let registry = match app_handle.try_state::<Arc<crate::core::formats::FormatRegistry>>() {
            Some(r) => r,
            None => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    b"FormatRegistry not found".to_vec(),
                )
            }
        };

        if let Some(format) = registry.inner().detect(&physical_path) {
            if let Some(provider) = registry.inner().get_provider(&format.name) {
                if let Some(preview_cap) = provider.preview() {
                    let preview_result: AppResult<(Vec<u8>, String)> = tauri::async_runtime::block_on(async {
                        preview_cap.generate_preview(&physical_path, &asset.id).await
                    });

                    if let Ok((data, mime)) = preview_result {
                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, mime)
                            .header(header::CONTENT_LENGTH, data.len().to_string())
                            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                            .header(header::CACHE_CONTROL, "public, max-age=3600")
                            .body(data)
                            .unwrap_or_else(|_| Response::default());
                    }
                }
            }
        }
        // Fallback to serving original file if preview extraction fails or is not supported
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

/// Helper to decode URI, and test if `type=thumb` is present.
///
/// # Arguments
/// * `uri`: The URI to parse.
///
/// # Returns
/// A tuple containing the asset ID and an optional string indicating the requested type.
///
/// # Examples
/// ```no_run
/// use crate::delivery::protocols::asset::parse_asset_uri;
///
/// let (asset_id, req_type) = parse_asset_uri("asset://localhost/12345?type=thumb");
/// assert_eq!(asset_id, "12345");
/// assert_eq!(req_type, Some("thumb".to_string()));
/// ```
fn parse_asset_uri(uri: &str) -> (String, Option<String>) {
    let prefix = "asset://localhost/";
    let fallback = "asset://";

    let path_with_query = if let Some(rest) = uri.strip_prefix(prefix) {
        rest
    } else if let Some(rest) = uri.strip_prefix(fallback) {
        rest
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

    let req_type = query_part.and_then(|q| {
        if q.contains("type=thumb") {
            Some("thumb".to_string())
        } else if q.contains("type=preview") {
            Some("preview".to_string())
        } else if q.contains("type=glb") {
            Some("glb".to_string())
        } else {
            None
        }
    });

    (decoded_id, req_type)
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

    let file = match tokio::fs::File::open(path).await {
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

    let _file_size = metadata.len();

    // Simplistic MIME detection. Use fallback octet setup if unfound.
    serve_file(path, range).await
}

