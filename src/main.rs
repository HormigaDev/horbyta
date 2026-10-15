use axum::{Router, middleware, routing::get};
use sea_orm::Database;
use tracing_subscriber;

use horbyta::{controllers, dtos::user::UserDto, env::Config, middlewares, state::AppState, swagger};

fn main() {
    // Load configuration before starting runtime threads. Exported variables win.
    if let Err(error) = dotenvy::dotenv() {
        if !error.not_found() {
            panic!("CRITICAL: Could not load .env. Check its syntax and permissions.");
        }
    }
    tracing_subscriber::fmt::init();

    tokio::runtime::Runtime::new()
        .expect("CRITICAL: Could not initialize the async runtime.")
        .block_on(run());
}

async fn run() {
    let config = Config::init();

    let db = Database::connect(&config.database_url)
        .await
        .expect("CRITICAL: Could not connect to Postgres. Verify DATABASE_URL.");

    horbyta::migrations::require_current(&db)
        .await
        .expect("Ejecute cargo run --locked --bin migrate antes de iniciar Horbyta.");

    let state = AppState::new(db, config.clone())
        .await
        .expect("CRITICAL: Failed to initialize AppState.");

    let delivery_state = state.clone();
    tokio::spawn(async move {
        horbyta::services::scope_delivery::run(delivery_state).await;
    });

    let api_routes = Router::new()
        .route(
            "/health",
            get(|user: UserDto| async move {
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
        .nest("/scope", controllers::scope::scope_controller())
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
        .route("/ws/scope/client", get(controllers::conversation_ws::client))
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not install Ctrl+C handler.");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Could not install SIGTERM handler.")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
