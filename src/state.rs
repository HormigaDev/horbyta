use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::{dtos::user::UserDto, env::Config};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub user_cache: Cache<Uuid, UserDto>,
    pub config: Config,
}

impl AppState {
    pub fn new(db: DatabaseConnection, config: Config) -> Self {
        let user_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(10_000)
            .build();

        Self {
            db: Arc::new(db),
            user_cache,
            config,
        }
    }
}
