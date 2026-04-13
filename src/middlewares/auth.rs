use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{constants::USER_AUTH_COOKIE_NAME, dtos::user::UserDto, entities::users, state::AppState};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

impl FromRequestParts<AppState> for UserDto {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar.get(USER_AUTH_COOKIE_NAME).map(|c| c.value()).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "No session", "type": "Unauthorized"})),
            )
        })?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid Token", "type": "JwtError"})),
            )
        })?;

        let user_id = token_data.claims.sub;

        // Fast path: Caché
        if let Some(user) = state.user_cache.get(&user_id).await {
            return Ok(user);
        }

        // Slow path: Base de datos
        let user_model = users::Entity::find_by_id(user_id)
            .one(state.db.as_ref())
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "DB Error", "type": "Database"})),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid Token", "type": "Unauthorized"})),
                )
            })?;

        let user_dto = UserDto {
            id: user_model.id,
            name: user_model.name,
            email: user_model.email,
            created_at: user_model.created_at,
            updated_at: user_model.updated_at,
        };

        state.user_cache.insert(user_id, user_dto.clone()).await;

        Ok(user_dto)
    }
}
