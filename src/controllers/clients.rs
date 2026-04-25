use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::{
            CreateClientDto, SaveClientCredentialsDto, UpdateClientDto, UpdateClientStatusDto, client_filter_schema,
        },
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::clients::ClientService,
    state::AppState,
};

async fn search_clients(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    client_filter_schema().validate_payload(&payload)?;

    let (clients, total_items) = ClientService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": clients,
        "total": total_items
    })))
}

async fn get_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(client))
}

async fn create_client(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(client))
}

async fn update_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(client))
}

async fn update_client_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateClientStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::update_client_status(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(client))
}

async fn delete_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ClientService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({"message": "Cliente eliminado"})))
}

async fn get_client_auth_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let auth_status = ClientService::get_client_auth_status(state.db.as_ref(), user.id, id).await?;
    Ok(Json(auth_status))
}

async fn activate_client_credentials(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<SaveClientCredentialsDto>,
) -> Result<impl IntoResponse, AppError> {
    ClientService::save_client_credentials(state.db.as_ref(), user.id, id, payload).await?;
    let auth_status = ClientService::get_client_auth_status(state.db.as_ref(), user.id, id).await?;
    Ok(Json(auth_status))
}

async fn activate_client_magic_token(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let result =
        ClientService::generate_client_magic_token(state.db.as_ref(), state.config.time_to_exp, user.id, id).await?;
    Ok(Json(result))
}

pub fn clients_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_clients))
        .route("/", post(create_client))
        .route("/{id}/auth", get(get_client_auth_status))
        .route("/{id}/auth/credentials", post(activate_client_credentials))
        .route("/{id}/auth/magic-token", post(activate_client_magic_token))
        .route(
            "/{id}",
            get(get_client)
                .put(update_client)
                .patch(update_client_status)
                .delete(delete_client),
        )
}
