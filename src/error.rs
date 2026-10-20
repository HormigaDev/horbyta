use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde_json::json;
use std::{borrow::Cow, fmt};

use crate::error_codes as codes;

/// Error del catálogo: código estable para el frontend y mensaje base con `$1`, `$2`…
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    pub code: &'static str,
    pub message: &'static str,
}

impl ErrorCode {
    /// Rellena los `$n` del mensaje con `params`, que también viajan en la respuesta.
    pub fn with<I, P>(self, params: I) -> ErrorDetail
    where
        I: IntoIterator<Item = P>,
        P: ToString,
    {
        let params: Vec<String> = params.into_iter().map(|p| p.to_string()).collect();
        ErrorDetail {
            code: Cow::Borrowed(self.code),
            message: fill_params(self.message, &params),
            params,
        }
    }
}

/// Lo que recibe el cliente: `code` para traducir, `message` legible y `params` variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDetail {
    pub code: Cow<'static, str>,
    pub message: String,
    pub params: Vec<String>,
}

impl ErrorDetail {
    /// Mensaje ya redactado (p. ej. validación) con el código y los datos que lo describen.
    pub fn new(code: ErrorCode, message: impl Into<String>, params: Vec<String>) -> Self {
        Self {
            code: Cow::Borrowed(code.code),
            message: message.into(),
            params,
        }
    }

    /// Solo para herramientas de consola (migraciones, seed): no llega a la interfaz.
    pub fn plain(message: impl Into<String>) -> Self {
        Self {
            code: Cow::Borrowed("UNCATALOGED"),
            message: message.into(),
            params: Vec::new(),
        }
    }
}

impl From<ErrorCode> for ErrorDetail {
    fn from(code: ErrorCode) -> Self {
        code.with(Vec::<String>::new())
    }
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl PartialEq<str> for ErrorDetail {
    fn eq(&self, other: &str) -> bool {
        self.message == other
    }
}

impl PartialEq<&str> for ErrorDetail {
    fn eq(&self, other: &&str) -> bool {
        self.message == *other
    }
}

impl PartialEq<String> for ErrorDetail {
    fn eq(&self, other: &String) -> bool {
        &self.message == other
    }
}

/// Sustituye `$n` por el parámetro n (de mayor a menor para que `$1` no pise `$10`).
fn fill_params(template: &str, params: &[String]) -> String {
    let mut message = template.to_owned();
    for (index, value) in params.iter().enumerate().rev() {
        message = message.replace(&format!("${}", index + 1), value);
    }
    message
}

/// Reconoce un mensaje del catálogo (también con `$n`) y devuelve su código y sus datos.
pub fn match_catalog(message: &str) -> Option<ErrorDetail> {
    codes::ALL.iter().find_map(|code| {
        extract_params(code.message, message).map(|params| ErrorDetail {
            code: Cow::Borrowed(code.code),
            message: message.to_owned(),
            params,
        })
    })
}

fn extract_params(template: &str, message: &str) -> Option<Vec<String>> {
    // Trozos literales entre marcadores `$n`, en orden.
    let mut literals = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('$') {
        let digits = rest[start + 1..].chars().take_while(char::is_ascii_digit).count();
        if digits == 0 {
            break;
        }
        literals.push(&rest[..start]);
        rest = &rest[start + 1 + digits..];
    }
    literals.push(rest);
    if literals.len() == 1 {
        return (template == message).then(Vec::new);
    }
    let mut params = Vec::new();
    let mut tail = message.strip_prefix(literals[0])?;
    for (index, literal) in literals.iter().enumerate().skip(1) {
        let last = index == literals.len() - 1;
        let end = if last {
            if !tail.ends_with(literal) || tail.len() < literal.len() {
                return None;
            }
            tail.len() - literal.len()
        } else if literal.is_empty() {
            return None;
        } else {
            tail.find(literal)?
        };
        if end == 0 {
            return None;
        }
        params.push(tail[..end].to_owned());
        tail = &tail[end + literal.len()..];
    }
    Some(params)
}

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
    BadRequest(ErrorDetail),

    #[error("{0}")]
    Unauthorized(ErrorDetail),

    #[error("{0}")]
    Forbidden(ErrorDetail),

    #[error("{0}")]
    NotFound(ErrorDetail),

    #[error("{0}")]
    Conflict(ErrorDetail),

    #[error("{0}")]
    Unprocessable(ErrorDetail),

    #[error("{0}")]
    Locked(ErrorDetail),

    #[error("{0}")]
    NotImplemented(ErrorDetail),

    #[error("{0}")]
    Unavailable(ErrorDetail),

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
        let (status, type_str, detail): (StatusCode, &str, ErrorDetail) = match self {
            AppError::Database(_) | AppError::Time(_) | AppError::Header(_) | AppError::InternalError => {
                tracing::error!("{:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    codes::INTERNAL_ERROR.into(),
                )
            }
            AppError::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "jwt_error",
                codes::AUTH_SESSION_EXPIRED.into(),
            ),
            AppError::Hash(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "hash_error",
                codes::AUTH_CREDENTIALS_PROCESSING_FAILED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::AlreadyEnabled) => (
                StatusCode::CONFLICT,
                "two_factor_auth_already_enabled",
                codes::TWO_FACTOR_ALREADY_ENABLED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::NotEnabled) => (
                StatusCode::BAD_REQUEST,
                "two_factor_auth_not_enabled",
                codes::TWO_FACTOR_NOT_ENABLED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::SetupNotStarted) => (
                StatusCode::BAD_REQUEST,
                "two_factor_auth_setup_not_started",
                codes::TWO_FACTOR_SETUP_NOT_STARTED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::InvalidCode) => (
                StatusCode::UNAUTHORIZED,
                "invalid_two_factor_auth_code",
                codes::TWO_FACTOR_CODE_INVALID.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::CodeRequired) => (
                StatusCode::UNAUTHORIZED,
                "two_factor_auth_code_required",
                codes::TWO_FACTOR_CODE_REQUIRED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::ReplayedCode) => (
                StatusCode::UNAUTHORIZED,
                "replayed_two_factor_auth_code",
                codes::TWO_FACTOR_CODE_REPLAYED.into(),
            ),
            AppError::Totp2Fa(Totp2FaError::InvalidSecret) => {
                tracing::error!("invalid two-factor authentication secret");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "invalid_two_factor_auth_secret",
                    codes::TWO_FACTOR_PROCESSING_FAILED.into(),
                )
            }
            AppError::Totp2Fa(Totp2FaError::QrGeneration) => {
                tracing::error!("failed to generate two-factor authentication qr code");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "two_factor_auth_qr_generation",
                    codes::TWO_FACTOR_QR_FAILED.into(),
                )
            }
            AppError::BadRequest(detail) => (StatusCode::BAD_REQUEST, "bad_request", detail),
            AppError::Unauthorized(detail) => (StatusCode::UNAUTHORIZED, "unauthorized", detail),
            AppError::Forbidden(detail) => (StatusCode::FORBIDDEN, "forbidden", detail),
            AppError::NotFound(detail) => (StatusCode::NOT_FOUND, "not_found", detail),
            AppError::Conflict(detail) => (StatusCode::CONFLICT, "conflict", detail),
            AppError::Unprocessable(detail) => (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable_entity", detail),
            AppError::Locked(detail) => (StatusCode::from_u16(423).unwrap(), "locked", detail),
            AppError::NotImplemented(detail) => (StatusCode::NOT_IMPLEMENTED, "not_implemented", detail),
            AppError::Unavailable(detail) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", detail),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_requests",
                codes::RATE_LIMITED.into(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("{:?}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_server_error",
                    codes::INTERNAL_ERROR.into(),
                )
            }
        };

        error_response(status, type_str, &detail)
    }
}

/// Cuerpo de error para extractores que responden sin `AppError` (sesión de usuario y cliente).
pub fn error_body(type_str: &str, code: ErrorCode) -> Json<serde_json::Value> {
    Json(json!({ "error": code.message, "type": type_str, "code": code.code, "params": [] }))
}

/// Cuerpo común de error: `error` (texto), `type`, `code` (para traducir) y `params`.
pub fn error_response(status: StatusCode, type_str: &str, detail: &ErrorDetail) -> Response {
    let body = json!({ "error": detail.message, "type": type_str, "code": detail.code, "params": detail.params });
    (status, Json(body)).into_response()
}

impl From<sea_orm::DbErr> for AppError {
    fn from(error: sea_orm::DbErr) -> Self {
        let message = error.to_string();
        // Reglas de la base (`raise exception 'prefijo: mensaje'`): se reconocen por el catálogo.
        for (prefix, fallback) in [
            ("task_status: ", codes::TASK_STATUS_INVALID),
            ("scope_conflict: ", codes::SCOPE_CONFLICT),
        ] {
            if let Some((_, detail)) = message.split_once(prefix) {
                let text = detail.lines().next().unwrap_or(fallback.message);
                let detail = match_catalog(text).unwrap_or_else(|| ErrorDetail::new(fallback, text, Vec::new()));
                return Self::Conflict(detail);
            }
        }
        if message.contains("uq_task_statuses_name") {
            return Self::Conflict(codes::TASK_STATUS_NAME_TAKEN.into());
        }
        if message.contains("tasks_status_id_fkey") {
            return Self::Conflict(codes::TASK_STATUS_IN_USE.into());
        }
        if message.contains("invoice_has_payments") {
            return Self::Conflict(codes::INVOICE_VOID_HAS_PAYMENTS.into());
        }
        if message.contains("payment_tampering_detected") {
            return Self::Conflict(codes::PAYMENT_IMMUTABLE.into());
        }
        if message.contains(codes::CHANGE_REQUEST_ASSESSED_TEXT_LOCKED.message) {
            return Self::Conflict(codes::CHANGE_REQUEST_ASSESSED_TEXT_LOCKED.into());
        }
        if message.contains("scope_") && message.contains("foreign key") {
            return Self::Conflict(preserved_record(&message).into());
        }
        Self::Database(error)
    }
}

/// Un registro que forma parte de un acuerdo no se borra; el mensaje dice cuál y qué hacer.
fn preserved_record(message: &str) -> ErrorCode {
    let table = message
        .split_once("on table \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(table, _)| table);
    match table {
        Some("tasks") => codes::SCOPE_TASK_HAS_AGREEMENT,
        Some("projects") => codes::SCOPE_PROJECT_HAS_AGREEMENT,
        Some("change_requests") => codes::SCOPE_CHANGE_REQUEST_HAS_HISTORY,
        Some("milestones") => codes::SCOPE_MILESTONE_HAS_AGREEMENT,
        Some("clients") => codes::SCOPE_CLIENT_HAS_AGREEMENT,
        Some("attachments") => codes::SCOPE_ATTACHMENT_HAS_AGREEMENT,
        Some("users") => codes::SCOPE_USER_HAS_AGREEMENT,
        _ => codes::SCOPE_RECORD_HAS_AGREEMENT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigos_unicos_y_parametros_consecutivos() {
        let mut seen = std::collections::HashSet::new();
        for code in codes::ALL {
            assert!(seen.insert(code.code), "código repetido {}", code.code);
            let count = (1..=9).take_while(|n| code.message.contains(&format!("${n}"))).count();
            assert!(
                !code.message.contains(&format!("${}", count + 1)),
                "{}: $n no consecutivos",
                code.code
            );
        }
    }

    #[test]
    fn rellena_y_reconoce_parametros() {
        let detail = codes::PAYMENT_REFUND_EXCEEDS_BALANCE.with(["10.00", "4.50"]);
        assert_eq!(
            detail.message,
            "refund amount (10.00) exceeds the available balance (4.50)"
        );
        let matched = match_catalog(&detail.message).unwrap();
        assert_eq!(
            (matched.code.as_ref(), matched.params),
            ("PAYMENT_REFUND_EXCEEDS_BALANCE", detail.params)
        );
        let category = match_catalog("Debe existir al menos un estado de la categoría todo.").unwrap();
        assert_eq!(
            (category.code.as_ref(), category.params),
            ("TASK_STATUS_CATEGORY_REQUIRED", vec!["todo".to_owned()])
        );
        assert!(match_catalog("mensaje desconocido").is_none());
    }

    #[test]
    fn todos_los_mensajes_de_la_base_tienen_codigo() {
        // Cada `raise exception 'prefijo: …'` de las migraciones debe reconocerse por el catálogo.
        for (version, sql) in crate::migrations::up_scripts() {
            for prefix in ["scope_conflict: ", "task_status: "] {
                for chunk in sql.split(prefix).skip(1) {
                    let text = chunk.split('\'').next().unwrap().replace('%', "X");
                    assert!(
                        match_catalog(&text).is_some(),
                        "migración {version}: sin código para «{text}»"
                    );
                }
            }
        }
    }

    #[test]
    fn registros_con_acuerdo_dicen_que_hacer() {
        let message = "update or delete on table \"tasks\" violates foreign key constraint \"scope_task_links_task_id_fkey\" on table \"scope_task_links\"";
        assert_eq!(preserved_record(message).code, "SCOPE_TASK_HAS_AGREEMENT");
    }
}
