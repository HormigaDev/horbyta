pub mod controllers;
pub mod dtos;
pub mod entities;
pub mod env;
pub mod error;
pub mod middlewares;
pub mod services;
pub mod state;
pub mod utils;

use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use sea_orm::Database;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use crate::{env::Config, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::init();

    let db = Database::connect(&config.database_url)
        .await
        .expect("CRÍTICO: No se pudo conectar a Postgres. Verifica DATABASE_URL.");

    let state = AppState::new(db, config.clone())
        .await
        .expect("CRÍTICO: No se pudo inicializar AppState.");

    let api_routes = Router::new()
        .route(
            "/health",
            get(|user: crate::dtos::user::UserDto| async move {
                axum::Json(serde_json::json!({
                    "status": "ok",
                    "message": "HormigaPM API v1 Operativa",
                    "user": user
                }))
            }),
        )
        .nest("/auth", controllers::auth::auth_controller())
        .nest("/chats", controllers::chats::chats_controller())
        .nest("/projects", controllers::projects::projects_controller())
        .nest("/categories", controllers::categories::categories_controller())
        .nest("/milestones", controllers::milestones::milestones_controller())
        .nest("/tasks", controllers::tasks::tasks_controller())
        .nest("/tags", controllers::tags::tags_controller())
        .nest("/users", controllers::users::users_controller())
        .with_state(state.clone());

    let ws_routes = Router::new()
        .route("/", get(controllers::websocket::websocket_handler))
        .with_state(state.clone());

    let app = Router::new().nest("/api/v1", api_routes).nest("/ws", ws_routes);
    let app = if config.is_production {
        app
    } else {
        let allowed_origin =
            HeaderValue::from_str(&config.frontend_origin).expect("CRÍTICO: FRONTEND_ORIGIN inválido.");

        app.layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([ACCEPT, AUTHORIZATION, CONTENT_TYPE])
                .allow_credentials(true),
        )
    };

    let listener = tokio::net::TcpListener::bind(&config.app_uri).await.unwrap();

    println!("Servidor corriendo en http(s)://{}", config.app_uri);
    axum::serve(listener, app).await.unwrap();
}
