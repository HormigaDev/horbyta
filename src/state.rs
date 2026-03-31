use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::time::Duration;
use uuid::Uuid;

use crate::{dtos::UserDto, env::Config};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
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
            db,
            user_cache,
            config,
        }
    }
}
