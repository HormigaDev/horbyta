use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),

    #[error("jwt error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("hash validation error")]
    Hash(argon2::password_hash::Error),

    #[error("system clock error")]
    Time(#[from] std::time::SystemTimeError),

    #[error("http header parse error")]
    Header(#[from] axum::http::header::InvalidHeaderValue),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Unprocessable(String),

    #[error("{0}")]
    Locked(String),

    #[error("{0}")]
    NotImplemented(String),

    #[error("{0}")]
    Unavailable(String),

    #[error("internal error")]
    InternalError,

    #[error("{0}")]
    Internal(String),
}

pub type ApiError = AppError;

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        AppError::Hash(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, type_str, error_message) = match self {
            AppError::Database(_) | AppError::Time(_) | AppError::Header(_) | AppError::InternalError => {
                tracing::error!("{:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "internal server error".to_string(),
                )
            }
            AppError::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "jwt_error",
                "invalid or expired token".to_string(),
            ),
            AppError::Hash(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hash_error",
                "failed to process credentials".to_string(),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable_entity", msg),
            AppError::Locked(msg) => (StatusCode::from_u16(423).unwrap(), "locked", msg),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, "not_implemented", msg),
            AppError::Unavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error", msg),
        };

        (status, Json(json!({ "error": error_message, "type": type_str }))).into_response()
    }
}
