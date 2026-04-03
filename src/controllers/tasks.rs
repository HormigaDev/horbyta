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
        query::QueryPayload,
        task::{CreateTaskDto, UpdateTaskDto, task_filter_schema},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::tasks::TaskService,
    state::AppState,
};

async fn view_tasks(user: UserDto, service: TaskService) -> Result<impl IntoResponse, AppError> {
    let view_payload = service.view(user.id).await?;
    Ok(Json(view_payload))
}

async fn search_tasks(
    user: UserDto,
    service: TaskService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    task_filter_schema().validate_payload(&payload)?;

    let (tasks, total_items) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": tasks,
        "total": total_items
    })))
}

async fn get_task(
    user: UserDto,
    service: TaskService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let task = service.get_by_id(user.id, id).await?;
    Ok(Json(task))
}

async fn create_task(
    user: UserDto,
    service: TaskService,
    ValidatedJson(payload): ValidatedJson<CreateTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let task = service.create(user.id, payload).await?;
    Ok(Json(task))
}

async fn update_task(
    user: UserDto,
    service: TaskService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let task = service.update(user.id, id, payload).await?;
    Ok(Json(task))
}

async fn delete_task(
    user: UserDto,
    service: TaskService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({ "message": "Tarea eliminada" })))
}

pub fn tasks_controller() -> Router<AppState> {
    Router::new()
        .route("/view", get(view_tasks))
        .route("/search", get(search_tasks))
        .route("/", post(create_task))
        .route("/{id}", get(get_task).put(update_task).delete(delete_task))
}
