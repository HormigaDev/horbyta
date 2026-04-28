use moka::future::Cache;
use object_store::{ObjectStore, aws::AmazonS3Builder, local::LocalFileSystem};
use sea_orm::DatabaseConnection;
use std::time::Duration;
use std::{path::Path, sync::Arc};
use uuid::Uuid;

use crate::{dtos::client::ClientAuthDto, dtos::user::UserDto, env::Config, error::AppError};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub user_cache: Cache<Uuid, UserDto>,
    pub client_cache: Cache<Uuid, ClientAuthDto>,
    pub config: Config,
    pub storage: Arc<dyn ObjectStore>,
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

        Ok(Self {
            db,
            user_cache,
            client_cache,
            config,
            storage,
        })
    }
}
