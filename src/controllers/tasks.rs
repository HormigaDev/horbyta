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

async fn view_tasks(State(state): State<AppState>, user: UserDto) -> Result<impl IntoResponse, AppError> {
    let view_payload = TaskService::view(state.db.as_ref(), user.id).await?;
    Ok(Json(view_payload))
}

async fn search_tasks(
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

async fn get_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let task = TaskService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(task))
}

async fn create_task(
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

async fn update_task(
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

async fn update_task_status(
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

async fn delete_task(
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
