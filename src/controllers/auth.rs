use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    constants::{CLIENT_AUTH_COOKIE_NAME, USER_AUTH_COOKIE_NAME},
    dtos::{
        auth::{ClientMagicTokenLogin, LoginPayload},
        client::ClientAuthDto,
        user::UserDto,
    },
    entities::{client_auth, clients, users},
    error::AppError,
    middlewares::{auth::Claims, client_auth::ClientClaims, validator::ValidatedJson},
    state::AppState,
    utils::crypto::{compare_password, generate_jwt, verify_magic_link_token},
};

async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_model = users::Entity::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    compare_password(payload.password, user_model.password_hash)?;

    let claims = Claims {
        sub: user_model.id,
        exp: get_expiration_time(state.config.time_to_exp)?,
    };

    let token = crate::utils::crypto::generate_jwt(&claims, &state.config.jwt_secret)?;
    let cookie = build_auth_cookie(USER_AUTH_COOKIE_NAME, token, state.config.is_production);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Autenticación exitosa",
        "user": {
            "id": user_model.id,
            "name": user_model.name,
            "email": user_model.email,
        }
    }));

    Ok((StatusCode::OK, headers, body))
}

async fn login_client(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let client_model = client_auth::Entity::find()
        .filter(client_auth::Column::Email.eq(payload.email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    if let Some(password_hash) = &client_model.password_hash {
        compare_password(payload.password, password_hash.clone())?;
        return handle_client_login_success(client_model, &state).await;
    }

    Err(AppError::Unauthorized("invalid credentials".to_string()))
}

async fn login_magic_token(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ClientMagicTokenLogin>,
) -> Result<impl IntoResponse, AppError> {
    let hash = verify_magic_link_token(&payload.magic_token)?;

    let client_model = client_auth::Entity::find()
        .filter(client_auth::Column::MagicLinkHash.eq(hash))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("invalid credentials".to_string()))?;

    if let Some(exp) = client_model.magic_link_exp {
        if exp.with_timezone(&Utc) < Utc::now() {
            return Err(AppError::Unauthorized("expired token".to_string()));
        }
        return handle_client_login_success(client_model, &state).await;
    }

    Err(AppError::Unauthorized("invalid credentials".to_string()))
}

async fn register() -> Result<impl IntoResponse, AppError> {
    Err::<axum::Json<()>, AppError>(AppError::NotImplemented(
        "registration is disabled, personal-use system".to_string(),
    ))
}

async fn logout(_user: UserDto) -> Result<impl IntoResponse, AppError> {
    let cookie = Cookie::build((USER_AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Logout exitoso"
    }));

    Ok((StatusCode::OK, headers, body))
}

async fn logout_client(_client: ClientAuthDto) -> Result<impl IntoResponse, AppError> {
    let cookie = Cookie::build((CLIENT_AUTH_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .build();

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

fn build_auth_cookie<'a>(name: &'a str, token: String, is_production: bool) -> Cookie<'a> {
    Cookie::build((name, token))
        .path("/")
        .http_only(true)
        .secure(is_production)
        .same_site(SameSite::Strict)
        .build()
}

async fn client_me(client: ClientAuthDto) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json!({
        "id": client.id,
        "client_id": client.client_id,
        "user_id": client.user_id,
        "name": client.name,
        "email": client.email,
    })))
}

async fn handle_client_login_success(
    client_model: client_auth::Model,
    state: &AppState,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    let client_profile = client_model
        .find_related(clients::Entity)
        .one(state.db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("client not found".to_string()))?;

    let claims = ClientClaims {
        sub: client_model.id,
        exp: get_expiration_time(state.config.time_to_exp)?,
    };

    let token = generate_jwt(&claims, &state.config.jwt_secret_client)?;

    let cookie = build_auth_cookie(CLIENT_AUTH_COOKIE_NAME, token, state.config.is_production);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Autenticación exitosa",
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
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/logout-client", post(logout_client))
        .route("/client-me", get(client_me))
}
