use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        query::QueryPayload,
        task::{CreateTaskDto, UpdateTaskDto, UpdateTaskStatusDto, task_filter_schema},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::tasks::TaskService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/tasks/view",
    tag = "Tasks",
    responses(
        (status = 200, description = "Task view summary", body = crate::dtos::task::TaskViewDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn view_tasks(State(state): State<AppState>, user: UserDto) -> Result<impl IntoResponse, AppError> {
    let view_payload = TaskService::view(state.db.as_ref(), user.id).await?;
    Ok(Json(view_payload))
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/search",
    tag = "Tasks",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of tasks"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_tasks(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    task_filter_schema().validate_payload(&payload)?;

    let (tasks, total_items) = TaskService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": tasks,
        "total": total_items
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}",
    tag = "Tasks",
    params(("id" = Uuid, Path, description = "Task id")),
    responses(
        (status = 200, description = "Task detail", body = crate::dtos::task::TaskDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
pub(crate) async fn get_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let task = TaskService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(task))
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/",
    tag = "Tasks",
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Task created", body = crate::dtos::task::TaskDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_task(
    State(state): State<AppState>,
    user: UserDto,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<CreateTaskDto>(
        &state,
        request,
        "multipart request must include a payload field with task json",
    )
    .await?;
    let task =
        TaskService::create_with_markdown_files(state.db.as_ref(), state.storage.as_ref(), user.id, payload, files)
            .await?;

    Ok(Json(task))
}

#[utoipa::path(
    put,
    path = "/api/v1/tasks/{id}",
    tag = "Tasks",
    params(("id" = Uuid, Path, description = "Task id")),
    request_body(content_type = "multipart/form-data", description = "Multipart form with a `payload` JSON field and optional files"),
    responses(
        (status = 200, description = "Task updated", body = crate::dtos::task::TaskDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
pub(crate) async fn update_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<UpdateTaskDto>(
        &state,
        request,
        "multipart request must include a payload field with task json",
    )
    .await?;
    let task =
        TaskService::update_with_markdown_files(state.db.as_ref(), state.storage.as_ref(), user.id, id, payload, files)
            .await?;
    Ok(Json(task))
}

#[utoipa::path(
    patch,
    path = "/api/v1/tasks/{id}",
    tag = "Tasks",
    params(("id" = Uuid, Path, description = "Task id")),
    request_body = UpdateTaskStatusDto,
    responses(
        (status = 200, description = "Task status updated"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
pub(crate) async fn update_task_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTaskStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    TaskService::change_status(state.db.as_ref(), user.id, id, payload).await?;
    Ok((
        StatusCode::OK,
        Json(json!({"message": "Tarea actualizada correctamente"})),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    tag = "Tasks",
    params(("id" = Uuid, Path, description = "Task id")),
    responses(
        (status = 200, description = "Task deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
pub(crate) async fn delete_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    TaskService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Tarea eliminada" })))
}

pub fn tasks_controller() -> Router<AppState> {
    Router::new()
        .route("/view", get(view_tasks))
        .route("/search", get(search_tasks))
        .route("/", post(create_task))
        .route(
            "/{id}",
            get(get_task)
                .put(update_task)
                .patch(update_task_status)
                .delete(delete_task),
        )
}
