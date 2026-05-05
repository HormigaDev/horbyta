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

use crate::{
    constants::CLIENT_AUTH_COOKIE_NAME,
    entities::{client_auth, clients},
};
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
            let is_active = clients::Entity::find_by_id(client.client_id)
                .one(state.db.as_ref())
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "db error", "type": "database"})),
                    )
                })?
                .is_some_and(|client_profile| client_profile.status == "active");

            if !is_active {
                state.client_cache.invalidate(&client_id).await;
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "client access is not active", "type": "forbidden"})),
                ));
            }

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

        if client_profile.status != "active" {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({"error": "client access is not active", "type": "forbidden"})),
            ));
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::USER_AUTH_COOKIE_NAME, env::Config, state::AppState};
    use axum::body::Body;
    use axum::extract::FromRequestParts;
    use axum::http::Request;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::time::Duration;

    fn test_config() -> Config {
        Config {
            database_url: "mock://db".to_string(),
            jwt_secret: "user-secret".to_string(),
            jwt_secret_client: "client-secret".to_string(),
            app_uri: "127.0.0.1:0".to_string(),
            frontend_origin: "http://localhost:9000".to_string(),
            is_production: false,
            time_to_exp: Duration::from_secs(3600),
            s3_bucket: None,
            s3_endpoint: None,
            s3_region: None,
            s3_access_key: None,
            s3_secret_key: None,
            upload_dir: Some("/tmp/horbyta-test-storage".to_string()),
            allow_registration: true,
        }
    }

    async fn test_state() -> AppState {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        AppState::new(db, test_config()).await.expect("state should build")
    }

    #[tokio::test]
    async fn freelancer_cookie_cannot_authenticate_client_endpoint() {
        let state = test_state().await;
        let request = Request::builder()
            .header("cookie", format!("{USER_AUTH_COOKIE_NAME}=user-token"))
            .body(Body::empty())
            .expect("valid request");
        let (mut parts, _) = request.into_parts();

        let result = ClientAuthDto::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err((StatusCode::UNAUTHORIZED, _))));
    }
}
