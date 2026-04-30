use dashmap::DashMap;
use moka::future::Cache;
use moka::sync::Cache as SyncCache;
use object_store::{ObjectStore, aws::AmazonS3Builder, local::LocalFileSystem};
use sea_orm::DatabaseConnection;
use std::net::IpAddr;
use std::sync::{Arc, atomic::AtomicUsize};
use std::time::Duration;
use std::{path::Path, sync::atomic::AtomicU32};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    dtos::client::ClientAuthDto, dtos::ui_state::UiStateEntryDto, dtos::user::UserDto, env::Config,
    error::AppError, services::notifications::NotificationEvent,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub user_cache: Cache<Uuid, UserDto>,
    pub client_cache: Cache<Uuid, ClientAuthDto>,
    pub ui_state_cache: Cache<String, Vec<UiStateEntryDto>>,
    pub config: Config,
    pub storage: Arc<dyn ObjectStore>,
    pub notification_bus: broadcast::Sender<NotificationEvent>,
    // Rate limiters — fixed-window counters (TTL = window duration)
    pub rate_limit_auth: SyncCache<IpAddr, Arc<AtomicU32>>,
    pub rate_limit_charts: SyncCache<Uuid, Arc<AtomicU32>>,
    pub rate_limit_uploads: SyncCache<Uuid, Arc<AtomicU32>>,
    pub rate_limit_magic_token: SyncCache<Uuid, Arc<AtomicU32>>,
    // WebSocket connection counter per user
    pub ws_connections: Arc<DashMap<Uuid, AtomicUsize>>,
}

impl AppState {
    pub async fn new(db: DatabaseConnection, config: Config) -> Result<Self, AppError> {
        let db = Arc::new(db);

        let user_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(5_000)
            .build();
        let client_cache = Cache::builder()
            .time_to_idle(Duration::from_secs(15 * 60))
            .max_capacity(10_000)
            .build();
        // UI state cache — keyed by `<user_id>:<scope>`; entries are invalidated on
        // every write/delete so a generous TTL is fine.
        let ui_state_cache = Cache::builder()
            .time_to_live(Duration::from_secs(5 * 60))
            .max_capacity(20_000)
            .build();

        let storage: Arc<dyn ObjectStore> = if let Some(bucket) = &config.s3_bucket {
            tracing::info!("Initializing S3 storage driver for bucket: {}", bucket);

            let s3 = AmazonS3Builder::new()
                .with_region(config.s3_region.as_ref().unwrap_or(&String::from("us-east-1")))
                .with_bucket_name(bucket)
                .with_access_key_id(config.s3_access_key.as_deref().unwrap_or_default())
                .with_secret_access_key(config.s3_secret_key.as_deref().unwrap_or_default())
                .with_endpoint(config.s3_endpoint.as_deref().unwrap_or_default())
                .with_allow_http(true)
                .build()
                .map_err(|e| AppError::Internal(format!("failed to init s3: {}", e)))?;

            Arc::new(s3)
        } else {
            let upload_dir = config.upload_dir.as_deref().unwrap_or("./data/storage");
            let path = Path::new(upload_dir);

            std::fs::create_dir_all(path)
                .map_err(|e| AppError::Internal(format!("failed to create upload dir: {}", e)))?;

            let absolute_path = path
                .canonicalize()
                .map_err(|e| AppError::Internal(format!("failed to resolve absolute path: {}", e)))?;

            tracing::info!(
                "Initializing Local storage driver at absolute path: {:?}",
                absolute_path
            );

            let local = LocalFileSystem::new_with_prefix(absolute_path)
                .map_err(|e| AppError::Internal(format!("failed to init local storage: {}", e)))?;

            Arc::new(local)
        };
        let (notification_bus, _) = broadcast::channel(1_024);

        // 5 attempts per minute per IP (login, register, magic-token login)
        let rate_limit_auth = SyncCache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(50_000)
            .build();

        // 60 requests per minute per user (dashboard + all chart endpoints)
        let rate_limit_charts = SyncCache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(10_000)
            .build();

        // 10 uploads per minute per user
        let rate_limit_uploads = SyncCache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(10_000)
            .build();

        // 5 magic-token generations per minute per user
        let rate_limit_magic_token = SyncCache::builder()
            .time_to_live(Duration::from_secs(60))
            .max_capacity(10_000)
            .build();

        let ws_connections = Arc::new(DashMap::new());

        Ok(Self {
            db,
            user_cache,
            client_cache,
            ui_state_cache,
            config,
            storage,
            notification_bus,
            rate_limit_auth,
            rate_limit_charts,
            rate_limit_uploads,
            rate_limit_magic_token,
            ws_connections,
        })
    }
}
