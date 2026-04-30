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
        change_request::{
            AssessChangeRequestDto, AssignChangeRequestTaskDto, ClientApproveChangeRequestDto,
            ClientCancelChangeRequestDto, ClientCreateChangeRequestDto, ClientDeclineChangeRequestDto,
            ClientUpdateChangeRequestDto, CreateChangeRequestDto, UpdateChangeRequestDto, change_request_filter_schema,
            client_change_request_filter_schema,
        },
        client::ClientAuthDto,
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::{change_requests::ChangeRequestService, notifications::NotificationService},
    state::AppState,
};

async fn search_change_requests(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    change_request_filter_schema().validate_payload(&payload)?;

    let (items, total_items) = ChangeRequestService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total_items
    })))
}

async fn client_search_change_requests(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    client_change_request_filter_schema().validate_payload(&payload)?;

    let (items, total_items) =
        ChangeRequestService::client_search(state.db.as_ref(), client.user_id, client.client_id, payload.clone())
            .await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total_items
    })))
}

async fn get_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(change_request))
}

async fn client_get_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::get_client_by_id(state.db.as_ref(), client.user_id, client.client_id, id).await?;
    Ok(Json(change_request))
}

async fn create_change_request(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(change_request))
}

async fn client_create_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedJson(payload): ValidatedJson<ClientCreateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::create_by_client(state.db.as_ref(), client.user_id, client.client_id, payload).await?;
    if let Err(error) = NotificationService::notify_client_change_request_created(
        &state,
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        tracing::error!("Failed to create change request notification: {}", error);
    }
    Ok(Json(change_request))
}

async fn client_update_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientUpdateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::client_update_spec(state.db.as_ref(), client.user_id, client.client_id, id, payload)
            .await?;
    if let Err(error) = NotificationService::notify_client_change_request_updated(
        &state,
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        tracing::error!("Failed to create change request update notification: {}", error);
    }
    Ok(Json(change_request))
}

async fn update_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(change_request))
}

async fn assign_change_request_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AssignChangeRequestTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::assign_task_for_approved(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(change_request))
}

async fn submit_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::submit(state.db.as_ref(), user.id, id).await?;
    Ok(Json(change_request))
}

async fn assess_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AssessChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::assess(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(change_request))
}

async fn mark_change_request_implemented(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::mark_implemented(state.db.as_ref(), user.id, id).await?;
    Ok(Json(change_request))
}

async fn cancel_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::cancel(state.db.as_ref(), user.id, id).await?;
    Ok(Json(change_request))
}

async fn delete_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ChangeRequestService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Solicitud de cambio eliminada" })))
}

async fn client_approve_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientApproveChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::client_approve(state.db.as_ref(), client.user_id, client.client_id, id, payload).await?;
    if let Err(error) = NotificationService::notify_client_change_request_decision(
        &state,
        client.user_id,
        client.client_id,
        &change_request,
        true,
    )
    .await
    {
        tracing::error!("Failed to create change request approval notification: {}", error);
    }
    Ok(Json(change_request))
}

async fn client_decline_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientDeclineChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::client_decline(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        id,
        payload.decline_reason,
    )
    .await?;
    if let Err(error) = NotificationService::notify_client_change_request_decision(
        &state,
        client.user_id,
        client.client_id,
        &change_request,
        false,
    )
    .await
    {
        tracing::error!("Failed to create change request decline notification: {}", error);
    }
    Ok(Json(change_request))
}

async fn client_cancel_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientCancelChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::client_cancel(state.db.as_ref(), client.user_id, client.client_id, id, payload).await?;
    if let Err(error) = NotificationService::notify_client_change_request_cancelled(
        &state,
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        tracing::error!("Failed to create change request cancellation notification: {}", error);
    }
    Ok(Json(change_request))
}

pub fn change_requests_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_change_requests))
        .route("/client/search", get(client_search_change_requests))
        .route("/", post(create_change_request))
        .route("/client", post(client_create_change_request))
        .route(
            "/client/{id}",
            get(client_get_change_request).patch(client_update_change_request),
        )
        .route(
            "/client/{id}/approve",
            axum::routing::patch(client_approve_change_request),
        )
        .route(
            "/client/{id}/decline",
            axum::routing::patch(client_decline_change_request),
        )
        .route(
            "/client/{id}/cancel",
            axum::routing::patch(client_cancel_change_request),
        )
        .route(
            "/{id}",
            get(get_change_request)
                .put(update_change_request)
                .delete(delete_change_request),
        )
        .route("/{id}/task", axum::routing::patch(assign_change_request_task))
        .route("/{id}/submit", axum::routing::patch(submit_change_request))
        .route("/{id}/assess", axum::routing::patch(assess_change_request))
        .route(
            "/{id}/implemented",
            axum::routing::patch(mark_change_request_implemented),
        )
        .route("/{id}/cancel", axum::routing::patch(cancel_change_request))
}
