pub mod constants;
pub mod controllers;
pub mod dtos;
pub mod entities;
pub mod env;
pub mod error;
pub mod macros;
pub mod middlewares;
pub mod services;
pub mod state;
pub mod swagger;
pub mod utils;

use axum::{Router, middleware, routing::get};
use sea_orm::Database;
use tracing_subscriber;

use crate::{env::Config, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::init();

    let db = Database::connect(&config.database_url)
        .await
        .expect("CRITICAL: Could not connect to Postgres. Verify DATABASE_URL.");

    let state = AppState::new(db, config.clone())
        .await
        .expect("CRITICAL: Failed to initialize AppState.");

    let api_routes = Router::new()
        .route(
            "/health",
            get(|user: crate::dtos::user::UserDto| async move {
                axum::Json(serde_json::json!({
                    "status": "ok",
                    "message": "Horbyta API v1 Operativa",
                    "user": user
                }))
            }),
        )
        .nest("/auth", controllers::auth::auth_controller())
        .nest("/dashboard", controllers::dashboard::dashboard_controller())
        .nest("/attachments", controllers::attachments::attachments_controller())
        .nest(
            "/change-requests",
            controllers::change_requests::change_requests_controller(),
        )
        .nest("/projects", controllers::projects::projects_controller())
        .nest("/categories", controllers::categories::categories_controller())
        .nest("/milestones", controllers::milestones::milestones_controller())
        .nest("/tasks", controllers::tasks::tasks_controller())
        .nest("/tags", controllers::tags::tags_controller())
        .nest("/invoices", controllers::invoices::invoices_controller())
        .nest("/payments", controllers::payments::payments_controller())
        .nest("/time-entries", controllers::time_entries::time_entries_controller())
        .nest("/users", controllers::users::users_controller())
        .nest("/clients", controllers::clients::clients_controller())
        .nest("/activities", controllers::activities::activities_controller())
        .with_state(state.clone());

    let ws_routes = Router::new()
        .route("/ws", get(controllers::websocket::websocket_handler))
        .route("/ws/", get(controllers::websocket::websocket_handler))
        .with_state(state.clone());

    let app = Router::new().nest("/api/v1", api_routes).merge(ws_routes);
    let app = if config.is_production {
        app
    } else {
        app.merge(swagger::docs_router())
    };
    let app = app
        .layer(middlewares::origin::cors_layer(&config))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            middlewares::origin::enforce_origin_policy,
        ));

    let listener = tokio::net::TcpListener::bind(&config.app_uri).await.unwrap();

    println!("Server running at http(s)://{}", config.app_uri);
    axum::serve(listener, app).await.unwrap();
}
