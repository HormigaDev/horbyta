use axum::{
    Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::{CreateClientDto, UpdateClientDto, UpdateClientStatusDto, client_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::clients::ClientService,
    state::AppState,
};

async fn search_clients(
    user: UserDto,
    service: ClientService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    client_filter_schema().validate_payload(&payload)?;

    let (clients, total_items) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": clients,
        "total": total_items
    })))
}

async fn get_client(
    user: UserDto,
    service: ClientService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = service.get_by_id(user.id, id).await?;
    Ok(Json(client))
}

async fn create_client(
    user: UserDto,
    service: ClientService,
    ValidatedJson(payload): ValidatedJson<CreateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = service.create(user.id, payload).await?;
    Ok(Json(client))
}

async fn update_client(
    user: UserDto,
    service: ClientService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = service.update(user.id, id, payload).await?;
    Ok(Json(client))
}

async fn update_client_status(
    user: UserDto,
    service: ClientService,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateClientStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = service.update_client_status(user.id, id, payload).await?;
    Ok(Json(client))
}

async fn delete_client(
    user: UserDto,
    service: ClientService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({"message": "Cliente eliminado"})))
}

pub fn clients_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_clients))
        .route("/", post(create_client))
        .route(
            "/{id}",
            get(get_client)
                .put(update_client)
                .patch(update_client_status)
                .delete(delete_client),
        )
}
