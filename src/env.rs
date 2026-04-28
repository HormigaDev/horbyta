use std::{env, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_secret_client: String,
    pub app_uri: String,
    pub frontend_origin: String,
    pub is_production: bool,
    pub time_to_exp: Duration,

    // Storage
    pub s3_bucket: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,

    pub upload_dir: Option<String>,
}

impl Config {
    pub fn init() -> Self {
        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            database_url: env::var("DATABASE_URL").expect("CRÍTICO: Falta DATABASE_URL en las variables de entorno."),
            jwt_secret: env::var("JWT_SECRET").expect("CRÍTICO: Falta JWT_SECRET en las variables de entorno"),
            jwt_secret_client: env::var("JWT_SECRET_CLIENT")
                .expect("CRÍTICO: Falta JWT_SECRET_CLIENT en las variables de entorno"),
            app_uri: env::var("APP_URI").unwrap_or_else(|_| "127.0.0.1:8890".to_string()),
            frontend_origin: env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            is_production: environment == "production",
            time_to_exp: duration_str::parse(
                env::var("TIME_TO_EXP_SESSION")
                    .expect("CRÍTICO: Falta TIME_TO_EXP_SESSION en las variables de entorno"),
            )
            .expect("CRÍTICO: Formato incorrecto para la variable de entorno TIME_TO_EXP_SESSION"),

            s3_bucket: match env::var("S3_BUCKET") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
            s3_endpoint: match env::var("S3_ENDPOINT") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
            s3_region: match env::var("S3_REGION") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
            s3_access_key: match env::var("S3_ACCESS_KEY") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
            s3_secret_key: match env::var("S3_SECRET_KEY") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
            upload_dir: match env::var("UPLOAD_DIR") {
                Ok(var) => Some(var),
                Err(_) => None,
            },
        }
    }
}
