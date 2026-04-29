use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{EntityTrait, ModelTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{constants::CLIENT_AUTH_COOKIE_NAME, entities::{client_auth, clients}};
use crate::{dtos::client::ClientAuthDto, state::AppState};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientClaims {
    pub sub: Uuid,
    pub exp: usize,
}

impl FromRequestParts<AppState> for ClientAuthDto {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar.get(CLIENT_AUTH_COOKIE_NAME).map(|c| c.value()).ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "no session", "type": "unauthorized"})),
            )
        })?;

        let token_data = decode::<ClientClaims>(
            token,
            &DecodingKey::from_secret(state.config.jwt_secret_client.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "invalid token", "type": "jwt_error"})),
            )
        })?;

        let client_id = token_data.claims.sub;

        if let Some(client) = state.client_cache.get(&client_id).await {
            return Ok(client);
        }

        let client_model = client_auth::Entity::find_by_id(client_id)
            .one(state.db.as_ref())
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "db error", "type": "database"})),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "invalid token", "type": "unauthorized"})),
                )
            })?;

        let client_profile = client_model
            .find_related(clients::Entity)
            .one(state.db.as_ref())
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "db error", "type": "database"})),
                )
            })?
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "client not found", "type": "unauthorized"})),
                )
            })?;

        let client_dto = ClientAuthDto {
            id: client_model.id,
            client_id: client_model.client_id,
            user_id: client_model.user_id,
            name: client_profile.name,
            email: client_profile.email,
            created_at: client_model.created_at,
            updated_at: client_model.updated_at,
        };

        state.client_cache.insert(client_id, client_dto.clone()).await;

        Ok(client_dto)
    }
}
