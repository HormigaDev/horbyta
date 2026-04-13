use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    dtos::{client::ClientAuthDto, user::UserDto},
    env::Config,
    services::ai::AiService,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub user_cache: Cache<Uuid, UserDto>,
    pub client_cache: Cache<Uuid, ClientAuthDto>,
    pub config: Config,
    pub ai_service: Arc<tokio::sync::Mutex<Option<AiService>>>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: Config) -> Result<Self, crate::error::AppError> {
        let db = Arc::new(db);

        let user_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(5_000)
            .build();
        let client_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(10_000)
            .build();

        let ai_service = match AiService::connect(&config.ai_service_url, db.clone()).await {
            Ok(service) => Some(service),
            Err(error) => {
                tracing::warn!(
                    ai_service_url = %config.ai_service_url,
                    error = %error,
                    "AI service unavailable at startup, continuing in degraded mode"
                );
                None
            }
        };

        Ok(Self {
            db,
            user_cache,
            client_cache,
            config,
            ai_service: Arc::new(tokio::sync::Mutex::new(ai_service)),
        })
    }
}
