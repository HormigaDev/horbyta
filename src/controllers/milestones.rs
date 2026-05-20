use axum::{
    Json, Router,
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::TransactionTrait;
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::ClientAuthDto,
        milestone::{
            CreateMilestoneDto, MilestoneStatus, UpdateMilestoneDto, client_milestone_filter_schema,
            milestone_filter_schema,
        },
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedSearchQuery,
    services::{attachments::AttachmentService, milestones::MilestoneService, notifications::NotificationService},
    state::AppState,
    utils::notifications::publish_notification,
};

#[utoipa::path(
    get,
    path = "/api/v1/milestones/search",
    tag = "Milestones",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of milestones"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_milestones(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) = MilestoneService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/milestones/client/search-by-project/{id}",
    tag = "Milestones",
    params(
        ("id" = Uuid, Path, description = "Project id"),
        crate::dtos::query::SearchQuery
    ),
    responses(
        (status = 200, description = "Paginated list of client-visible milestones"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn client_search_milestones(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    client_milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) = MilestoneService::client_search_project_milestones(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        id,
        payload.clone(),
    )
    .await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/milestones/",
    tag = "Milestones",
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Milestone created", body = crate::dtos::milestone::ProjectMilestoneSummaryDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_milestone(
    State(state): State<AppState>,
    user: UserDto,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<CreateMilestoneDto>(
        &state,
        request,
        "multipart request must include a payload field with milestone json",
    )
    .await?;
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let milestone =
        MilestoneService::create_with_markdown_files(&tx, state.storage.as_ref(), user.id, payload, files).await?;

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &milestone.storage_keys).await;
        return Err(AppError::Database(error));
    }

    Ok(Json(milestone.value))
}

#[utoipa::path(
    put,
    path = "/api/v1/milestones/{id}",
    tag = "Milestones",
    params(("id" = Uuid, Path, description = "Milestone id")),
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Milestone updated", body = crate::dtos::milestone::ProjectMilestoneSummaryDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Milestone not found")
    )
)]
pub(crate) async fn update_milestone(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<UpdateMilestoneDto>(
        &state,
        request,
        "multipart request must include a payload field with milestone json",
    )
    .await?;
    let requested_status = payload.status.clone();
    let previous = MilestoneService::get_by_id(state.db.as_ref(), user.id, id).await?;
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let milestone =
        MilestoneService::update_with_markdown_files(&tx, state.storage.as_ref(), user.id, id, payload, files).await?;

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &milestone.storage_keys).await;
        return Err(AppError::Database(error));
    }

    let milestone = milestone.value;
    if previous.status != MilestoneStatus::Released.to_string() && requested_status == MilestoneStatus::Released {
        match NotificationService::notify_milestone_released(state.db.as_ref(), user.id, &milestone).await {
            Ok(notification) => publish_notification(&state, user.id, notification),
            Err(error) => {
                tracing::error!("Failed to create milestone notification: {}", error);
            }
        };
    }
    Ok(Json(milestone))
}

#[utoipa::path(
    delete,
    path = "/api/v1/milestones/{id}",
    tag = "Milestones",
    params(("id" = Uuid, Path, description = "Milestone id")),
    responses(
        (status = 200, description = "Milestone deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Milestone not found")
    )
)]
pub(crate) async fn delete_milestone(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    MilestoneService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Hito eliminado" })))
}

pub fn milestones_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_milestones))
        .route("/", post(create_milestone))
        .route("/client/search-by-project/{id}", get(client_search_milestones))
        .route("/{id}", axum::routing::put(update_milestone).delete(delete_milestone))
}
