use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::ClientAuthDto,
        project::{
            CreateProjectDto, UpdateProjectCategoryDto, UpdateProjectDto, UpdateProjectStatusDto,
            client_project_filter_schema, project_filter_schema,
        },
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::projects::ProjectService,
    state::AppState,
};

async fn search_projects(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    project_filter_schema().validate_payload(&payload)?;

    let (projects, total_items) = ProjectService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": projects,
        "total": total_items
    })))
}

async fn get_project(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(project))
}

async fn client_search_projects(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    client_project_filter_schema().validate_payload(&payload)?;

    let (projects, total_items) =
        ProjectService::client_search_projects(state.db.as_ref(), client.user_id, client.client_id, payload.clone())
            .await?;

    Ok(Json(json!({
        "payload": payload,
        "items": projects,
        "total": total_items
    })))
}

async fn client_get_project(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::get_client_project_by_id(state.db.as_ref(), client.user_id, client.client_id, id)
        .await?;

    Ok(Json(project))
}

async fn create_project(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateProjectDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(project))
}

async fn update_project(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(project))
}

async fn update_project_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::update_category(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(project))
}

async fn update_project_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProjectStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::update_status(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(project))
}

async fn delete_project(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ProjectService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Proyecto eliminado" })))
}

pub fn projects_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_projects))
        .route("/client-search", get(client_search_projects))
        .route("/", post(create_project))
        .route("/{id}", get(get_project).put(update_project).delete(delete_project))
        .route("/{id}/category", patch(update_project_category))
        .route("/{id}/status", patch(update_project_status))
        .route("/{id}/client", get(client_get_project))
}
