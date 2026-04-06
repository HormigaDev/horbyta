use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub app_uri: String,
    pub ai_service_url: String,
    pub is_production: bool,
}

impl Config {
    pub fn init() -> Self {
        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            database_url: env::var("DATABASE_URL").expect("CRÍTICO: Falta DATABASE_URL en las variables de entorno."),
            jwt_secret: env::var("JWT_SECRET").expect("CRÍTICO: Falta JWT_SECRET en las variables de entorno"),
            app_uri: env::var("APP_URI").unwrap_or_else(|_| "127.0.0.1:8890".to_string()),
            ai_service_url: env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://horbyta_ai:50051".to_string()),
            is_production: environment == "production",
        }
    }
}
