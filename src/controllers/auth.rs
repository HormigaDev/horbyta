use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    dtos::auth::LoginPayload,
    entities::users,
    error::AppError,
    middlewares::{auth::Claims, validator::ValidatedJson},
    state::AppState,
};

async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_model = users::Entity::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::Unauthorized("Credenciales inválidas".to_string()))?;

    let parsed_hash = PasswordHash::new(&user_model.password_hash)?;

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::Unauthorized("Credenciales inválidas".to_string()));
    }

    let time_now = SystemTime::now().duration_since(UNIX_EPOCH)?;

    let claims = Claims {
        sub: user_model.id,
        exp: time_now.as_secs() as usize + (60 * 60 * 24 * 7),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )?;

    let cookie = Cookie::build(("hormiga_auth", token))
        .path("/")
        .http_only(true)
        .secure(state.config.is_production)
        .same_site(SameSite::Strict)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse()?);

    let body = Json(json!({
        "message": "Autenticación exitosa",
        "user": {
            "id": user_model.id,
            "name": user_model.name,
            "email": user_model.email,
            "created_at": user_model.created_at,
            "updated_at": user_model.updated_at
        }
    }));

    Ok((StatusCode::OK, headers, body))
}

async fn register() -> Result<impl IntoResponse, AppError> {
    Err::<axum::Json<()>, AppError>(AppError::NotImplemented(
        "Registro deshabilitado. Sistema de uso personal.".to_string(),
    ))
}

pub fn auth_controller() -> Router<AppState> {
    let router = Router::new()
        .route("/login", post(login))
        .route("/register", post(register));

    router
}
