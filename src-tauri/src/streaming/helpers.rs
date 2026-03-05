use crate::core::error::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct StreamError(pub AppError);

impl IntoResponse for StreamError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self.0 {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB Error: {}", e),
            ),
            AppError::Transcoding(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Transcoding Error: {}", msg),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()),
        };

        tracing::error!("Streaming request failed ({}): {}", status, error_message);

        let body = Json(json!({ "error": error_message, "code": status.as_u16() }));
        (status, body).into_response()
    }
}

impl From<AppError> for StreamError {
    fn from(inner: AppError) -> Self {
        StreamError(inner)
    }
}

impl From<std::io::Error> for StreamError {
    fn from(err: std::io::Error) -> Self {
        StreamError(AppError::Io(err))
    }
}

impl From<String> for StreamError {
    fn from(msg: String) -> Self {
        StreamError(AppError::Generic(msg))
    }
}
