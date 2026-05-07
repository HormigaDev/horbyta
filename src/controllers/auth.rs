use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use cookie::time::{Duration as CookieDuration, OffsetDateTime};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, Set};
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    auth::totp_2fa::default_totp_policy,
    constants::{CLIENT_AUTH_COOKIE_NAME, USER_AUTH_COOKIE_NAME},
    dtos::{
        auth::{ClientLoginPayload, ClientMagicTokenLogin, LoginPayload, RegisterPayload},
        client::ClientAuthDto,
        user::UserDto,
    },
    entities::{client_auth, clients, users},
    error::{AppError, Totp2FaError},
    middlewares::{
        auth::Claims,
        client_auth::ClientClaims,
        rate_limit::{
            AUTH_RATE_LIMIT, TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT, check_ip_rate_limit, check_key_rate_limit,
            record_key_rate_limit_failure,
        },
        validator::ValidatedJson,
    },
    services::users::UserService,
    state::AppState,
    utils::crypto::{compare_password, generate_jwt, verify_magic_link_token},
};

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "User authenticated, sets auth cookie"),
        (status = 400, description = "Invalid payload"),
        (status = 401, description = "Invalid credentials"),
        (status = 429, description = "Too many login attempts")
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    check_ip_rate_limit(&state.rate_limit_auth, &headers, AUTH_RATE_LIMIT)?;

    let user_model = users::Entity::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    compare_password(payload.password, user_model.password_hash.clone())?;

    let two_factor_rate_limit_key = two_factor_rate_limit_key(user_model.id);
    if user_model.two_factor_auth_enabled {
        check_key_rate_limit(
            &state.rate_limit_totp_failures,
            &two_factor_rate_limit_key,
            TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT,
        )?;
    }

    let two_factor_result = UserService::verify_totp_for_login(
        state.db.as_ref(),
        user_model,
        &default_totp_policy(),
        payload.two_factor_auth_code,
        payload.two_factor_auth_backup_code,
    )
    .await;
    record_two_factor_failure(&state, &two_factor_rate_limit_key, &two_factor_result);
    let user_model = two_factor_result?;
    UserService::ensure_user_can_authenticate(&user_model)?;

    let claims = Claims {
        sub: user_model.id,
        exp: get_expiration_time(state.config.time_to_exp)?,
    };

    let token = crate::utils::crypto::generate_jwt(&claims, &state.config.jwt_secret)?;
    let cookie = build_auth_cookie(
        USER_AUTH_COOKIE_NAME,
        token,
        state.config.is_production,
        state.config.time_to_exp,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Authentication successful",
        "user": {
            "id": user_model.id,
            "name": user_model.name,
            "email": user_model.email,
            "role": user_model.role,
            "status": user_model.status,
        }
    }));

    Ok((StatusCode::OK, headers, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login-client",
    tag = "Auth",
    request_body = ClientLoginPayload,
    responses(
        (status = 200, description = "Client authenticated, sets client auth cookie"),
        (status = 401, description = "Invalid credentials"),
        (status = 429, description = "Too many login attempts")
    )
)]
pub(crate) async fn login_client(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<ClientLoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    check_ip_rate_limit(&state.rate_limit_auth, &headers, AUTH_RATE_LIMIT)?;

    let normalized_email = payload.email.trim().to_lowercase();
    let client_model = client_auth::Entity::find()
        .filter(client_auth::Column::UserId.eq(payload.user_id))
        .filter(client_auth::Column::Email.eq(normalized_email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    let password_hash = client_model
        .password_hash
        .clone()
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;
    compare_password(payload.password, password_hash)?;
    handle_client_login_success(client_model, &state).await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login-client-token",
    tag = "Auth",
    request_body = ClientMagicTokenLogin,
    responses(
        (status = 200, description = "Client authenticated via magic token"),
        (status = 401, description = "Invalid or expired magic token"),
        (status = 429, description = "Too many attempts")
    )
)]
pub(crate) async fn login_magic_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<ClientMagicTokenLogin>,
) -> Result<impl IntoResponse, AppError> {
    check_ip_rate_limit(&state.rate_limit_auth, &headers, AUTH_RATE_LIMIT)?;

    let hash = verify_magic_link_token(&payload.magic_token)?;

    let client_model = client_auth::Entity::find()
        .filter(client_auth::Column::UserId.eq(payload.user_id))
        .filter(client_auth::Column::MagicLinkHash.eq(hash))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    if let Some(exp) = client_model.magic_link_exp {
        if exp.with_timezone(&Utc) < Utc::now() {
            return Err(AppError::Unauthorized("expired token".to_string()));
        }

        let mut active_client_auth: client_auth::ActiveModel = client_model.clone().into();
        active_client_auth.magic_link_exp = Set(Some((Utc::now() - chrono::Duration::seconds(1)).fixed_offset()));
        let client_model = active_client_auth.update(state.db.as_ref()).await?;

        return handle_client_login_success(client_model, &state).await;
    }

    Err(AppError::Unauthorized("invalid credentials".to_string()))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/registration-status",
    tag = "Auth",
    responses(
        (status = 200, description = "Whether public registration is open")
    )
)]
pub(crate) async fn registration_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({ "open": state.config.allow_registration }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "User registered, sets auth cookie"),
        (status = 400, description = "Invalid payload"),
        (status = 409, description = "Email or username already taken"),
        (status = 423, description = "Registration is disabled"),
        (status = 429, description = "Too many attempts")
    )
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    check_ip_rate_limit(&state.rate_limit_auth, &headers, AUTH_RATE_LIMIT)?;

    if !state.config.allow_registration {
        return Err(AppError::Locked("registration is disabled".to_string()));
    }

    let user_model = UserService::register(state.db.as_ref(), payload).await?;

    let claims = Claims {
        sub: user_model.id,
        exp: get_expiration_time(state.config.time_to_exp)?,
    };

    let token = generate_jwt(&claims, &state.config.jwt_secret)?;
    let cookie = build_auth_cookie(
        USER_AUTH_COOKIE_NAME,
        token,
        state.config.is_production,
        state.config.time_to_exp,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Registro exitoso",
        "user": {
            "id": user_model.id,
            "name": user_model.name,
            "email": user_model.email,
            "role": user_model.role,
            "status": user_model.status,
        }
    }));

    Ok((StatusCode::CREATED, headers, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "User logged out, clears auth cookie"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn logout(State(state): State<AppState>, _user: UserDto) -> Result<impl IntoResponse, AppError> {
    let cookie = build_clear_auth_cookie(USER_AUTH_COOKIE_NAME, state.config.is_production);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Logout exitoso"
    }));

    Ok((StatusCode::OK, headers, body))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout-client",
    tag = "Auth",
    responses(
        (status = 200, description = "Client logged out, clears client cookie"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn logout_client(
    State(state): State<AppState>,
    _client: ClientAuthDto,
) -> Result<impl IntoResponse, AppError> {
    let cookie = build_clear_auth_cookie(CLIENT_AUTH_COOKIE_NAME, state.config.is_production);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Logout exitoso"
    }));

    Ok((StatusCode::OK, headers, body))
}

fn get_expiration_time(time_to_exp: Duration) -> Result<usize, AppError> {
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(time_now.as_secs() as usize + time_to_exp.as_secs() as usize)
}

fn build_auth_cookie<'a>(
    name: &'a str,
    token: String,
    is_production: bool,
    time_to_exp: Duration,
) -> Result<Cookie<'a>, AppError> {
    let duration = cookie_duration(time_to_exp)?;

    Ok(Cookie::build((name, token))
        .path("/")
        .http_only(true)
        .secure(is_production)
        .same_site(SameSite::Strict)
        .max_age(duration)
        .expires(OffsetDateTime::now_utc() + duration)
        .build())
}

fn build_clear_auth_cookie<'a>(name: &'a str, is_production: bool) -> Cookie<'a> {
    Cookie::build((name, ""))
        .path("/")
        .http_only(true)
        .secure(is_production)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::ZERO)
        .expires(OffsetDateTime::UNIX_EPOCH)
        .build()
}

fn cookie_duration(time_to_exp: Duration) -> Result<CookieDuration, AppError> {
    let seconds = i64::try_from(time_to_exp.as_secs())
        .map_err(|_| AppError::Internal("session duration is too large".to_string()))?;

    Ok(CookieDuration::seconds(seconds))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/client-me",
    tag = "Auth",
    responses(
        (status = 200, description = "Returns current authenticated client"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn client_me(client: ClientAuthDto) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({
        "id": client.id,
        "client_id": client.client_id,
        "user_id": client.user_id,
        "name": client.name,
        "email": client.email,
    })))
}

pub(crate) async fn handle_client_login_success(
    client_model: client_auth::Model,
    state: &AppState,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    let client_profile = client_model
        .find_related(clients::Entity)
        .one(state.db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("client not found".to_string()))?;

    if client_profile.status != "active" {
        return Err(AppError::Forbidden("client access is not active".to_string()));
    }

    let claims = ClientClaims {
        sub: client_model.id,
        exp: get_expiration_time(state.config.time_to_exp)?,
    };

    let token = generate_jwt(&claims, &state.config.jwt_secret_client)?;

    let cookie = build_auth_cookie(
        CLIENT_AUTH_COOKIE_NAME,
        token,
        state.config.is_production,
        state.config.time_to_exp,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Authentication successful",
        "client": {
            "id": client_model.id,
            "client_id": client_model.client_id,
            "user_id": client_model.user_id,
            "name": client_profile.name,
            "email": client_profile.email,
        }
    }));

    Ok((StatusCode::OK, headers, body))
}

pub fn auth_controller() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/login-client", post(login_client))
        .route("/login-client-token", post(login_magic_token))
        .route("/registration-status", get(registration_status))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/logout-client", post(logout_client))
        .route("/client-me", get(client_me))
}

fn two_factor_rate_limit_key(user_id: Uuid) -> String {
    format!("user-2fa:{user_id}")
}

fn record_two_factor_failure<T>(state: &AppState, key: &str, result: &Result<T, AppError>) {
    if matches!(
        result,
        Err(AppError::Totp2Fa(
            Totp2FaError::InvalidCode | Totp2FaError::ReplayedCode
        ))
    ) {
        record_key_rate_limit_failure(&state.rate_limit_totp_failures, key.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_cookie_uses_configured_session_duration() {
        let cookie = build_auth_cookie(
            USER_AUTH_COOKIE_NAME,
            "token".to_string(),
            false,
            Duration::from_secs(7 * 24 * 60 * 60),
        )
        .expect("cookie should build");

        assert_eq!(cookie.max_age(), Some(CookieDuration::days(7)));
        assert!(cookie.expires().is_some());
        assert!(cookie.http_only().unwrap_or(false));
        assert_eq!(cookie.path(), Some("/"));
    }

    #[test]
    fn clear_cookie_expires_immediately() {
        let cookie = build_clear_auth_cookie(USER_AUTH_COOKIE_NAME, false);

        assert_eq!(cookie.max_age(), Some(CookieDuration::ZERO));
        assert_eq!(
            cookie.expires(),
            Some(cookie::Expiration::DateTime(OffsetDateTime::UNIX_EPOCH))
        );
        assert!(cookie.http_only().unwrap_or(false));
        assert_eq!(cookie.path(), Some("/"));
    }
}
