use axum::{
    Json, Router,
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::ClientAuthDto,
        project::{
            CreateProjectDto, UpdateProjectBillableDto, UpdateProjectCategoryDto, UpdateProjectDto,
            UpdateProjectStatusDto, client_project_filter_schema, project_filter_schema,
        },
        project_billing::UpdateProjectBillingDto,
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
    let project =
        ProjectService::get_client_project_by_id(state.db.as_ref(), client.user_id, client.client_id, id).await?;

    Ok(Json(project))
}

async fn create_project(
    State(state): State<AppState>,
    user: UserDto,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<CreateProjectDto>(
        &state,
        request,
        "multipart request must include a payload field with project json",
    )
    .await?;
    let project =
        ProjectService::create_with_markdown_files(state.db.as_ref(), state.storage.as_ref(), user.id, payload, files)
            .await?;

    Ok(Json(project))
}

async fn update_project(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    request: Request,
) -> Result<impl IntoResponse, AppError> {
    let (payload, files) = crate::utils::multipart::parse_payload_request::<UpdateProjectDto>(
        &state,
        request,
        "multipart request must include a payload field with project json",
    )
    .await?;
    let project = ProjectService::update_with_markdown_files(
        state.db.as_ref(),
        state.storage.as_ref(),
        user.id,
        id,
        payload,
        files,
    )
    .await?;
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
    ValidatedJson(payload): ValidatedJson<UpdateProjectStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::update_status(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(project))
}

async fn update_project_billable(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectBillableDto>,
) -> Result<impl IntoResponse, AppError> {
    let project = ProjectService::update_billable(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(project))
}

async fn upsert_project_billing(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateProjectBillingDto>,
) -> Result<impl IntoResponse, AppError> {
    let billing = ProjectService::upsert_billing(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(billing))
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
        .route("/{id}/billable", patch(update_project_billable))
        .route("/{id}/billing", patch(upsert_project_billing))
        .route("/{id}/client", get(client_get_project))
}
