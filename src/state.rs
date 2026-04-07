use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::{dtos::user::UserDto, env::Config, services::ai::AiService};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub user_cache: Cache<Uuid, UserDto>,
    pub config: Config,
    pub ai_service: Arc<tokio::sync::Mutex<AiService>>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: Config) -> Result<Self, crate::error::AppError> {
        let db = Arc::new(db);

        let user_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(10_000)
            .build();

        let ai_service = AiService::connect(&config.ai_service_url, db.clone()).await?;

        Ok(Self {
            db,
            user_cache,
            config,
            ai_service: Arc::new(tokio::sync::Mutex::new(ai_service)),
        })
    }
}
