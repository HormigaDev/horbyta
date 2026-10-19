use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, Request, State},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        change_request::{
            AssessChangeRequestDto, AssignChangeRequestTaskDto, ClientApproveChangeRequestDto,
            ClientCancelChangeRequestDto, ClientCreateChangeRequestDto, ClientDeclineChangeRequestDto,
            ClientUpdateChangeRequestDto, CreateChangeRequestDto, ExpectedVersionDto, UpdateChangeRequestDto,
            change_request_filter_schema, client_change_request_filter_schema,
        },
        client::ClientAuthDto,
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::{
        attachments::AttachmentService, change_requests::ChangeRequestService, notifications::NotificationService,
        scope::ScopeService,
    },
    state::AppState,
    utils::notifications::publish_notification,
};

#[utoipa::path(
    get,
    path = "/api/v1/change-requests/search",
    tag = "Change Requests",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of change requests"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_change_requests(
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

#[utoipa::path(
    get,
    path = "/api/v1/change-requests/client/search",
    tag = "Change Requests",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of client-visible change requests"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn client_search_change_requests(
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

#[utoipa::path(
    get,
    path = "/api/v1/change-requests/{id}",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Change request detail", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn get_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request = ChangeRequestService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(change_request))
}

#[utoipa::path(
    get,
    path = "/api/v1/change-requests/client/{id}",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Client-visible change request detail", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn client_get_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let change_request =
        ChangeRequestService::get_client_by_id(state.db.as_ref(), client.user_id, client.client_id, id).await?;
    Ok(Json(change_request))
}

#[utoipa::path(
    post,
    path = "/api/v1/change-requests/",
    tag = "Change Requests",
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Change request created", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_change_request(
    State(state): State<AppState>,
    user: UserDto,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<CreateChangeRequestDto>(
        &state,
        request,
        "multipart request must include a payload field with change request json",
    )
    .await?;
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request =
        ChangeRequestService::create_with_markdown_files(&tx, state.storage.as_ref(), user.id, payload, files).await?;

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &change_request.storage_keys).await;
        return Err(AppError::Database(error));
    }

    if change_request.value.status != crate::dtos::change_request::ChangeRequestStatus::Draft {
        crate::services::project_conversation::publish_latest(&state, user.id, change_request.value.project_id, None)
            .await;
    }
    Ok(Json(change_request.value))
}

#[utoipa::path(
    post,
    path = "/api/v1/change-requests/client",
    tag = "Change Requests",
    request_body = ClientCreateChangeRequestDto,
    responses(
        (status = 200, description = "Client change request created", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn client_create_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedJson(payload): ValidatedJson<ClientCreateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, client.id, "client").await?;
    let change_request = ChangeRequestService::create_by_client(&tx, client.user_id, client.client_id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, client.user_id, change_request.project_id, None)
        .await;
    match NotificationService::notify_client_change_request_created(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        Ok(notification) => publish_notification(&state, client.user_id, notification),
        Err(error) => tracing::error!("Failed to create change request notification: {}", error),
    };
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/client/{id}",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = ClientUpdateChangeRequestDto,
    responses(
        (status = 200, description = "Client change request updated", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn client_update_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientUpdateChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, client.id, "client").await?;
    let change_request =
        ChangeRequestService::client_update_spec(&tx, client.user_id, client.client_id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, client.user_id, change_request.project_id, None)
        .await;
    match NotificationService::notify_client_change_request_updated(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        Ok(notification) => publish_notification(&state, client.user_id, notification),
        Err(error) => tracing::error!("Failed to create change request update notification: {}", error),
    };
    Ok(Json(change_request))
}

#[utoipa::path(
    put,
    path = "/api/v1/change-requests/{id}",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Change request updated", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn update_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<UpdateChangeRequestDto>(
        &state,
        request,
        "multipart request must include a payload field with change request json",
    )
    .await?;
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request =
        ChangeRequestService::update_with_markdown_files(&tx, state.storage.as_ref(), user.id, id, payload, files)
            .await?;

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &change_request.storage_keys).await;
        return Err(AppError::Database(error));
    }

    if change_request.value.status != crate::dtos::change_request::ChangeRequestStatus::Draft {
        crate::services::project_conversation::publish_latest(&state, user.id, change_request.value.project_id, None)
            .await;
    }
    Ok(Json(change_request.value))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/task",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = AssignChangeRequestTaskDto,
    responses(
        (status = 200, description = "Change request task assigned", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn assign_change_request_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AssignChangeRequestTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::assign_task_for_approved(&tx, user.id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/submit",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Change request submitted", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn submit_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ExpectedVersionDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::submit(&tx, user.id, id, payload.version).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/assess",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = AssessChangeRequestDto,
    responses(
        (status = 200, description = "Change request assessed", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn assess_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<AssessChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::assess(&tx, user.id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/implemented",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Change request marked implemented", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn mark_change_request_implemented(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ExpectedVersionDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::mark_implemented(&tx, user.id, id, payload.version).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/cancel",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Change request cancelled", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn cancel_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ExpectedVersionDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::cancel(&tx, user.id, id, payload.version).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/{id}/reopen",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = ExpectedVersionDto,
    responses(
        (status = 200, description = "Declined change request reopened for a new assessment", body = crate::dtos::change_request::ChangeRequestDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found"),
        (status = 409, description = "The request is not declined or its version changed")
    )
)]
pub(crate) async fn reopen_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ExpectedVersionDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    let change_request = ChangeRequestService::reopen(&tx, user.id, id, payload.version).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, user.id, change_request.project_id, None).await;
    Ok(Json(change_request))
}

#[utoipa::path(
    delete,
    path = "/api/v1/change-requests/{id}",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    responses(
        (status = 200, description = "Change request deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn delete_change_request(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, user.id, "professional").await?;
    ChangeRequestService::delete(&tx, user.id, id).await?;
    tx.commit().await?;
    Ok(Json(json!({ "message": "Solicitud de cambio eliminada" })))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/client/{id}/approve",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = ClientApproveChangeRequestDto,
    responses(
        (status = 200, description = "Client approved change request", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn client_approve_change_request(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientApproveChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, client.id, "client").await?;
    let (ip, source) = crate::middlewares::acceptance_ip::acceptance_ip(
        peer.ip(),
        &headers,
        &std::env::var("TRUSTED_PROXY_IPS").unwrap_or_default(),
    );
    tx.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        "SELECT set_config('horbyta.accepted_ip',$1,true),set_config('horbyta.ip_source',$2,true)",
        [ip.to_string().into(), source.into()],
    ))
    .await?;
    let change_request =
        ChangeRequestService::client_approve(&tx, client.user_id, client.client_id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, client.user_id, change_request.project_id, None)
        .await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/client/{id}/decline",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = ClientDeclineChangeRequestDto,
    responses(
        (status = 200, description = "Client declined change request", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn client_decline_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientDeclineChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, client.id, "client").await?;
    let change_request =
        ChangeRequestService::client_decline(&tx, client.user_id, client.client_id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, client.user_id, change_request.project_id, None)
        .await;
    Ok(Json(change_request))
}

#[utoipa::path(
    patch,
    path = "/api/v1/change-requests/client/{id}/cancel",
    tag = "Change Requests",
    params(("id" = Uuid, Path, description = "Change request id")),
    request_body = ClientCancelChangeRequestDto,
    responses(
        (status = 200, description = "Client cancelled change request", body = crate::dtos::change_request::ClientChangeRequestDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Change request not found")
    )
)]
pub(crate) async fn client_cancel_change_request(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<ClientCancelChangeRequestDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    ScopeService::actor(&tx, client.id, "client").await?;
    let change_request =
        ChangeRequestService::client_cancel(&tx, client.user_id, client.client_id, id, payload).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&state, client.user_id, change_request.project_id, None)
        .await;
    match NotificationService::notify_client_change_request_cancelled(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        &change_request,
    )
    .await
    {
        Ok(notification) => publish_notification(&state, client.user_id, notification),
        Err(error) => tracing::error!("Failed to create change request cancellation notification: {}", error),
    };
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
        .route("/{id}/reopen", axum::routing::patch(reopen_change_request))
}
