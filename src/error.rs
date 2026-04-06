use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Error de base de datos")]
    Database(#[from] sea_orm::DbErr),

    #[error("Error de JWT")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Error de validación de Hash")]
    Hash(argon2::password_hash::Error),

    #[error("Error del reloj del sistema")]
    Time(#[from] std::time::SystemTimeError),

    #[error("Error de parseo de cabeceras HTTP")]
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
    NotImplemented(String),

    #[error("{0}")]
    Unavailable(String),

    #[error("Error interno")]
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
                    "InternalServerError",
                    "Error interno del servidor".to_string(),
                )
            }
            AppError::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "JwtError",
                "Token inválido o expirado".to_string(),
            ),
            AppError::Hash(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "HashError",
                "Error procesando credenciales".to_string(),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BadRequest", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "Unauthorized", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "Forbidden", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NotFound", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict", msg),
            AppError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "UnprocessableEntity", msg),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, "NotImplemented", msg),
            AppError::Unavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "ServiceUnavailable", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", msg),
        };

        (status, Json(json!({ "error": error_message, "type": type_str }))).into_response()
    }
}
