use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum Totp2FaError {
    #[error("two-factor authentication is already enabled")]
    AlreadyEnabled,

    #[error("two-factor authentication is not enabled")]
    NotEnabled,

    #[error("two-factor authentication setup has not been started")]
    SetupNotStarted,

    #[error("invalid two-factor authentication code")]
    InvalidCode,

    #[error("two-factor authentication code is required")]
    CodeRequired,

    #[error("two-factor authentication code was already used")]
    ReplayedCode,

    #[error("invalid two-factor authentication secret")]
    InvalidSecret,

    #[error("failed to generate two-factor authentication qr code")]
    QrGeneration,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("database error")]
    Database(sea_orm::DbErr),

    #[error("jwt error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("hash validation error")]
    Hash(argon2::password_hash::Error),

    #[error("system clock error")]
    Time(#[from] std::time::SystemTimeError),

    #[error("http header parse error")]
    Header(#[from] axum::http::header::InvalidHeaderValue),

    #[error(transparent)]
    Totp2Fa(#[from] Totp2FaError),

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

    #[error("too many requests")]
    TooManyRequests,

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
            AppError::Totp2Fa(Totp2FaError::AlreadyEnabled) => (
                StatusCode::CONFLICT,
                "two_factor_auth_already_enabled",
                "two-factor authentication is already enabled".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::NotEnabled) => (
                StatusCode::BAD_REQUEST,
                "two_factor_auth_not_enabled",
                "two-factor authentication is not enabled".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::SetupNotStarted) => (
                StatusCode::BAD_REQUEST,
                "two_factor_auth_setup_not_started",
                "two-factor authentication setup has not been started".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::InvalidCode) => (
                StatusCode::UNAUTHORIZED,
                "invalid_two_factor_auth_code",
                "invalid two-factor authentication code".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::CodeRequired) => (
                StatusCode::UNAUTHORIZED,
                "two_factor_auth_code_required",
                "two-factor authentication code is required".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::ReplayedCode) => (
                StatusCode::UNAUTHORIZED,
                "replayed_two_factor_auth_code",
                "two-factor authentication code was already used".to_string(),
            ),
            AppError::Totp2Fa(Totp2FaError::InvalidSecret) => {
                tracing::error!("invalid two-factor authentication secret");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invalid_two_factor_auth_secret",
                    "failed to process two-factor authentication".to_string(),
                )
            }
            AppError::Totp2Fa(Totp2FaError::QrGeneration) => {
                tracing::error!("failed to generate two-factor authentication qr code");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "two_factor_auth_qr_generation",
                    "failed to generate two-factor authentication qr code".to_string(),
                )
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable_entity", msg),
            AppError::Locked(msg) => (StatusCode::from_u16(423).unwrap(), "locked", msg),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, "not_implemented", msg),
            AppError::Unavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", msg),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_requests",
                "too many requests, slow down".to_string(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("{:?}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": error_message, "type": type_str }))).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(error: sea_orm::DbErr) -> Self {
        let message = error.to_string();
        // Reglas de los estados de tarea, con el mismo formato que las del alcance.
        if let Some((_, detail)) = message.split_once("task_status: ") {
            return Self::Conflict(detail.lines().next().unwrap_or("Estado de tarea no válido.").to_owned());
        }
        if message.contains("uq_task_statuses_name") {
            return Self::Conflict("Ya existe un estado con ese nombre.".into());
        }
        if message.contains("tasks_status_id_fkey") {
            return Self::Conflict("El estado tiene tareas; muévelas a otro estado antes de eliminarlo.".into());
        }
        if let Some((_, detail)) = message.split_once("scope_conflict: ") {
            Self::Conflict(
                detail
                    .lines()
                    .next()
                    .unwrap_or("El acuerdo cambió; vuelva a leerlo.")
                    .to_owned(),
            )
        } else if message.contains("scope_") && message.contains("foreign key") {
            Self::Conflict("Este registro conserva acuerdos; ciérrelo o cancélelo sin eliminar su historial.".into())
        } else {
            Self::Database(error)
        }
    }
}
