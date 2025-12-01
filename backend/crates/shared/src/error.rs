use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        match self {
            DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            DomainError::Conflict(msg) => (StatusCode::CONFLICT, msg).into_response(),
            DomainError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}
