use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::TransactionTrait;
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        query::QueryPayload,
        scope::ClassifyTaskDto,
        task::{CreateTaskDto, UpdateTaskDto, UpdateTaskStatusDto, task_filter_schema},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::{attachments::AttachmentService, scope::ScopeService, tasks::TaskService},
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
    let mut payload = payload;
    let origin = payload.scope_origin.take();
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let task = TaskService::create_with_markdown_files(&tx, state.storage.as_ref(), user.id, payload, files).await?;

    // Con acuerdo vigente, toda tarea nueva declara su origen en la misma transacción:
    // dentro del alcance por defecto, o el extra elegido. Las subtareas lo heredan.
    let classified = async {
        let Some(project) = task.value.project_id.filter(|_| task.value.parent_id.is_none()) else {
            return Ok(());
        };
        if !ScopeService::has_active_agreement(&tx, project).await? {
            return Ok(());
        }
        let change_request_id = origin.and_then(|o| o.change_request_id);
        let dto = ClassifyTaskDto { change_request_id, reason: String::new() };
        ScopeService::classify(&tx, user.id, user.id, task.value.id, dto).await.map(|_| ())
    }
    .await;
    if let Err(error) = classified {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &task.storage_keys).await;
        return Err(error);
    }

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &task.storage_keys).await;
        return Err(AppError::Database(error));
    }

    Ok(Json(task.value))
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
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let task =
        TaskService::update_with_markdown_files(&tx, state.storage.as_ref(), user.id, id, payload, files).await?;

    if let Err(error) = tx.commit().await {
        AttachmentService::cleanup_storage_objects(state.storage.as_ref(), &task.storage_keys).await;
        return Err(AppError::Database(error));
    }

    Ok(Json(task.value))
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
    let tx = state.db.begin().await.map_err(AppError::Database)?;
    TaskService::change_status(&tx, user.id, id, payload).await?;
    tx.commit().await.map_err(AppError::Database)?;
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
