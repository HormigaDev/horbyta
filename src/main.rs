pub mod controllers;
pub mod dtos;
pub mod entities;
pub mod env;
pub mod error;
pub mod middlewares;
pub mod state;

use axum::{Router, routing::get};
use sea_orm::Database;

use crate::{env::Config, state::AppState};

#[tokio::main]
async fn main() {
    let config = Config::init();

    let db = Database::connect(&config.database_url)
        .await
        .expect("CRÍTICO: No se pudo conectar a Postgres. Verifica DATABASE_URL.");

    let state = AppState::new(db, config.clone());

    let api_routes = Router::new()
        .route(
            "/health",
            get(|_user: crate::dtos::UserDto| async {
                axum::Json(serde_json::json!({ "status": "ok", "message": "HormigaPM API v1 Operativa" }))
            }),
        )
        .nest("/auth", controllers::auth::auth_controller())
        .with_state(state);

    let app = Router::new().nest("/api/v1", api_routes);

    let listener = tokio::net::TcpListener::bind(&config.app_uri).await.unwrap();

    println!("Servidor corriendo en http(s)://{}", config.app_uri);
    axum::serve(listener, app).await.unwrap();
}
