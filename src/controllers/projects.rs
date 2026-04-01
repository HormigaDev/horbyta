use axum::{
    Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        project::{CreateProjectDto, UpdateProjectCategoryDto, UpdateProjectDto, project_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::projects::ProjectService,
    state::AppState,
};

async fn search_projects(
    user: UserDto,
    service: ProjectService,
    ValidatedJson(payload): ValidatedJson<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    project_filter_schema().validate_payload(&payload)?;

    let (projects, total_items) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": projects,
        "total": total_items
    })))
}

async fn get_project(
    user: UserDto,
    service: ProjectService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = service.get_by_id(user.id, id).await?;
    Ok(Json(project))
}

async fn create_project(
    user: UserDto,
    service: ProjectService,
    ValidatedJson(payload): ValidatedJson<CreateProjectDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = service.create(user.id, payload).await?;
    Ok(Json(project))
}

async fn update_project(
    user: UserDto,
    service: ProjectService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = service.update(user.id, id, payload).await?;
    Ok(Json(project))
}

async fn update_project_category(
    user: UserDto,
    service: ProjectService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = service.update_category(user.id, id, payload).await?;
    Ok(Json(project))
}

async fn delete_project(
    user: UserDto,
    service: ProjectService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({ "message": "Proyecto eliminado" })))
}

pub fn projects_controller() -> Router<AppState> {
    Router::new()
        .route("/search", post(search_projects))
        .route("/", post(create_project))
        .route("/:id", get(get_project).put(update_project).delete(delete_project))
        .route("/:id/category", patch(update_project_category))
}
