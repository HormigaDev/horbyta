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
    middlewares::{
        rate_limit::{MAGIC_TOKEN_RATE_LIMIT, check_user_rate_limit},
        validator::{ValidatedJson, ValidatedSearchQuery},
    },
    services::clients::ClientService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/clients/search",
    tag = "Clients",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of clients"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_clients(
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

#[utoipa::path(
    get,
    path = "/api/v1/clients/{id}",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    responses(
        (status = 200, description = "Client detail"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn get_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(client))
}

#[utoipa::path(
    post,
    path = "/api/v1/clients/",
    tag = "Clients",
    request_body = CreateClientDto,
    responses(
        (status = 200, description = "Client created"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_client(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(client))
}

#[utoipa::path(
    put,
    path = "/api/v1/clients/{id}",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    request_body = UpdateClientDto,
    responses(
        (status = 200, description = "Client updated"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn update_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateClientDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(client))
}

#[utoipa::path(
    patch,
    path = "/api/v1/clients/{id}",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    request_body = UpdateClientStatusDto,
    responses(
        (status = 200, description = "Client status updated"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn update_client_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateClientStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let client = ClientService::update_client_status(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(client))
}

#[utoipa::path(
    delete,
    path = "/api/v1/clients/{id}",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    responses(
        (status = 200, description = "Client deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn delete_client(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ClientService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({"message": "Cliente eliminado"})))
}

#[utoipa::path(
    get,
    path = "/api/v1/clients/{id}/auth",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    responses(
        (status = 200, description = "Client auth status", body = crate::dtos::client::ClientAuthStatusDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn get_client_auth_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let auth_status = ClientService::get_client_auth_status(state.db.as_ref(), user.id, id).await?;
    Ok(Json(auth_status))
}

#[utoipa::path(
    post,
    path = "/api/v1/clients/{id}/auth/credentials",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    request_body = SaveClientCredentialsDto,
    responses(
        (status = 200, description = "Client credentials enabled", body = crate::dtos::client::ClientAuthStatusDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn activate_client_credentials(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<SaveClientCredentialsDto>,
) -> Result<impl IntoResponse, AppError> {
    ClientService::save_client_credentials(state.db.as_ref(), user.id, id, payload).await?;
    let auth_status = ClientService::get_client_auth_status(state.db.as_ref(), user.id, id).await?;
    Ok(Json(auth_status))
}

#[utoipa::path(
    post,
    path = "/api/v1/clients/{id}/auth/magic-token",
    tag = "Clients",
    params(("id" = Uuid, Path, description = "Client id")),
    responses(
        (status = 200, description = "Generated magic link token", body = crate::dtos::client::ClientMagicLinkDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn activate_client_magic_token(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    check_user_rate_limit(&state.rate_limit_magic_token, user.id, MAGIC_TOKEN_RATE_LIMIT)?;
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
