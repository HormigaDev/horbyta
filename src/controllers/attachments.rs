use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::{
        HeaderMap, HeaderValue,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    dtos::{
        attachment::{AttachmentUploadResponseDto, AttachmentUploadScope},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedSearchQuery,
    services::attachments::AttachmentService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/attachments/project/{project_id}/search",
    tag = "Attachments",
    params(
        ("project_id" = Uuid, Path, description = "Project id"),
        crate::dtos::query::SearchQuery
    ),
    responses(
        (status = 200, description = "Paginated attachments for project"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_project_attachments(
    State(state): State<AppState>,
    user: UserDto,
    Path(project_id): Path<Uuid>,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    let response = AttachmentService::search_project(state.db.as_ref(), user.id, project_id, payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/task/{task_id}/search",
    tag = "Attachments",
    params(
        ("task_id" = Uuid, Path, description = "Task id"),
        crate::dtos::query::SearchQuery
    ),
    responses(
        (status = 200, description = "Paginated attachments for task"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_task_attachments(
    State(state): State<AppState>,
    user: UserDto,
    Path(task_id): Path<Uuid>,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    let response = AttachmentService::search_task(state.db.as_ref(), user.id, task_id, payload).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/{id}/raw",
    tag = "Attachments",
    params(("id" = Uuid, Path, description = "Attachment id")),
    responses(
        (status = 200, description = "Returns raw bytes of the attached file"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attachment not found")
    )
)]
pub(crate) async fn get_attachment_raw(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let file = AttachmentService::get_raw_file_for_cookies(&state, &jar, id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&file.mime_type)?);
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("sandbox; default-src 'none'; img-src 'self' data: blob:; style-src 'unsafe-inline'"),
    );
    headers.insert(CONTENT_DISPOSITION, HeaderValue::from_str(&file.content_disposition)?);

    Ok((headers, file.bytes))
}

#[utoipa::path(
    delete,
    path = "/api/v1/attachments/{id}",
    tag = "Attachments",
    params(("id" = Uuid, Path, description = "Attachment id")),
    responses(
        (status = 200, description = "Attachment deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attachment not found")
    )
)]
pub(crate) async fn delete_attachment(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let response =
        AttachmentService::delete_with_response(state.db.as_ref(), state.storage.as_ref(), user.id, id).await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/attachments/task/{task_id}/files",
    tag = "Attachments",
    params(("task_id" = Uuid, Path, description = "Task id")),
    request_body(content_type = "multipart/form-data", description = "Multipart form with one or more `files` parts"),
    responses(
        (status = 200, description = "Files attached to task"),
        (status = 400, description = "Invalid multipart payload or file too big"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn add_files_to_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(task_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<AttachmentUploadResponseDto>, AppError> {
    let response =
        AttachmentService::upload_files_to_scope(&state, user.id, AttachmentUploadScope::task(task_id), multipart)
            .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/attachments/project/{project_id}/files",
    tag = "Attachments",
    params(("project_id" = Uuid, Path, description = "Project id")),
    request_body(content_type = "multipart/form-data", description = "Multipart form with one or more `files` parts"),
    responses(
        (status = 200, description = "Files attached to project"),
        (status = 400, description = "Invalid multipart payload or file too big"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn add_files_to_project(
    State(state): State<AppState>,
    user: UserDto,
    Path(project_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<AttachmentUploadResponseDto>, AppError> {
    let response = AttachmentService::upload_files_to_scope(
        &state,
        user.id,
        AttachmentUploadScope::project(project_id),
        multipart,
    )
    .await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/usage",
    tag = "Attachments",
    responses(
        (status = 200, description = "Current storage usage", body = crate::dtos::attachment::AttachmentStorageUsageDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn get_storage_usage(
    State(state): State<AppState>,
    user: UserDto,
) -> Result<impl IntoResponse, AppError> {
    let usage = AttachmentService::get_user_storage_usage(state.db.as_ref(), user.id).await?;
    Ok(Json(usage))
}

pub fn attachments_controller() -> Router<AppState> {
    Router::new()
        .route("/usage", get(get_storage_usage))
        .route("/project/{project_id}/files", post(add_files_to_project))
        .route("/task/{task_id}/files", post(add_files_to_task))
        .route("/project/{project_id}/search", get(search_project_attachments))
        .route("/task/{task_id}/search", get(search_task_attachments))
        .route("/{id}/raw", get(get_attachment_raw))
        .route("/{id}", axum::routing::delete(delete_attachment))
}
