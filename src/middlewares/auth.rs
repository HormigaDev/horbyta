use std::str::FromStr;

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

use crate::{
    constants::USER_AUTH_COOKIE_NAME,
    dtos::user::{UserDto, UserRole, UserStatus},
    entities::users,
    state::AppState,
};

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
                Json(json!({"error": "no session", "type": "unauthorized"})),
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
                Json(json!({"error": "invalid token", "type": "jwt_error"})),
            )
        })?;

        let user_id = token_data.claims.sub;

        let user_model = users::Entity::find_by_id(user_id)
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

        let status = UserStatus::from_str(&user_model.status).unwrap_or(UserStatus::Inactive);
        match status {
            UserStatus::Active => {}
            UserStatus::Inactive => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "user not authenticated", "type": "unauthorized"})),
                ));
            }
            UserStatus::Blocked => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "account blocked", "type": "forbidden"})),
                ));
            }
        }

        let user_dto = UserDto {
            id: user_model.id,
            name: user_model.name,
            email: user_model.email,
            role: UserRole::from_str(&user_model.role).unwrap_or(UserRole::User),
            status,
            created_at: user_model.created_at,
            updated_at: user_model.updated_at,
        };

        state.user_cache.insert(user_id, user_dto.clone()).await;

        Ok(user_dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::CLIENT_AUTH_COOKIE_NAME, env::Config, state::AppState};
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
    async fn client_cookie_cannot_authenticate_freelancer_endpoint() {
        let state = test_state().await;
        let request = Request::builder()
            .header("cookie", format!("{CLIENT_AUTH_COOKIE_NAME}=client-token"))
            .body(Body::empty())
            .expect("valid request");
        let (mut parts, _) = request.into_parts();

        let result = UserDto::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err((StatusCode::UNAUTHORIZED, _))));
    }

    #[tokio::test]
    async fn freelancer_cannot_access_without_valid_jwt() {
        let state = test_state().await;
        let request = Request::builder()
            .header("cookie", format!("{USER_AUTH_COOKIE_NAME}=not-a-jwt"))
            .body(Body::empty())
            .expect("valid request");
        let (mut parts, _) = request.into_parts();

        let result = UserDto::from_request_parts(&mut parts, &state).await;

        assert!(matches!(result, Err((StatusCode::UNAUTHORIZED, _))));
    }
}
