use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
};
use sea_orm::TransactionTrait;
use uuid::Uuid;

use crate::{
    dtos::{
        task_status::{ReorderTaskStatusesDto, SaveTaskStatusDto},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::task_statuses::TaskStatusService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/task-statuses",
    tag = "Task statuses",
    responses(
        (status = 200, description = "Task statuses of the user, in board order", body = Vec<crate::dtos::task_status::TaskStatusDto>),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn list_task_statuses(
    State(state): State<AppState>,
    user: UserDto,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(TaskStatusService::list(state.db.as_ref(), user.id).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/task-statuses",
    tag = "Task statuses",
    request_body = SaveTaskStatusDto,
    responses(
        (status = 201, description = "Task status created", body = crate::dtos::task_status::TaskStatusDto),
        (status = 409, description = "Duplicated name")
    )
)]
pub(crate) async fn create_task_status(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(dto): ValidatedJson<SaveTaskStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let status = TaskStatusService::create(state.db.as_ref(), user.id, dto).await?;
    Ok((StatusCode::CREATED, Json(status)))
}

#[utoipa::path(
    put,
    path = "/api/v1/task-statuses/{id}",
    tag = "Task statuses",
    params(("id" = Uuid, Path, description = "Task status id")),
    request_body = SaveTaskStatusDto,
    responses(
        (status = 200, description = "Task status updated", body = crate::dtos::task_status::TaskStatusDto),
        (status = 409, description = "Duplicated name or category change of a status in use")
    )
)]
pub(crate) async fn update_task_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<SaveTaskStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(
        TaskStatusService::update(state.db.as_ref(), user.id, id, dto).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/task-statuses/{id}",
    tag = "Task statuses",
    params(("id" = Uuid, Path, description = "Task status id")),
    responses(
        (status = 204, description = "Task status deleted"),
        (status = 409, description = "Status in use or last of its category")
    )
)]
pub(crate) async fn delete_task_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    TaskStatusService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/task-statuses/order",
    tag = "Task statuses",
    request_body = ReorderTaskStatusesDto,
    responses((status = 200, description = "New order", body = Vec<crate::dtos::task_status::TaskStatusDto>))
)]
pub(crate) async fn reorder_task_statuses(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(dto): ValidatedJson<ReorderTaskStatusesDto>,
) -> Result<impl IntoResponse, AppError> {
    let tx = state.db.begin().await?;
    let statuses = TaskStatusService::reorder(&tx, user.id, dto).await?;
    tx.commit().await?;
    Ok(Json(statuses))
}

pub fn task_statuses_controller() -> Router<AppState> {
    Router::new()
        .route("/", get(list_task_statuses).post(create_task_status))
        .route("/order", put(reorder_task_statuses))
        .route("/{id}", put(update_task_status).delete(delete_task_status))
}
