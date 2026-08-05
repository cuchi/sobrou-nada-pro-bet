use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    /// The inner message is **never** sent to the client — it's logged only.
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, client_message) = match &self {
            // 4xx — safe to include the message (client needs it)
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            // 5xx — NEVER leak internals to the client
            AppError::Internal(detail) => {
                tracing::error!(%detail, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
        };
        (status, Json(json!({ "error": client_message }))).into_response()
    }
}
