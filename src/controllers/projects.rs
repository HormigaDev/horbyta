use axum::{
    Json, Router,
    body::to_bytes,
    extract::{FromRequest, Multipart, Path, Request, State},
    http::header::CONTENT_TYPE,
    response::IntoResponse,
    routing::{get, patch, post},
};
use sea_orm::TransactionTrait;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        attachment::CreateAttachmentDto,
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
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery, first_validation_error_message},
    services::{attachments::AttachmentService, projects::ProjectService},
    state::AppState,
};

const MAX_TOTAL_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
const MAX_JSON_BODY_BYTES: usize = 2 * 1024 * 1024;
const PAYLOAD_FIELD_NAME: &str = "payload";
const FILES_FIELD_NAME: &str = "files";

struct UploadedFile {
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

fn is_multipart_request(request: &Request) -> bool {
    request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|content_type| content_type.starts_with("multipart/form-data"))
        .unwrap_or(false)
}

async fn parse_project_payload_from_json(request: Request) -> Result<(CreateProjectDto, Vec<UploadedFile>), AppError> {
    let bytes = to_bytes(request.into_body(), MAX_JSON_BODY_BYTES)
        .await
        .map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

    let payload: CreateProjectDto =
        serde_json::from_slice(&bytes).map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

    payload
        .validate()
        .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

    Ok((payload, Vec::new()))
}

async fn parse_project_payload_from_multipart(
    state: &AppState,
    request: Request,
) -> Result<(CreateProjectDto, Vec<UploadedFile>), AppError> {
    let mut multipart = Multipart::from_request(request, state)
        .await
        .map_err(|_| AppError::Unprocessable("invalid multipart payload".to_string()))?;

    let mut payload: Option<CreateProjectDto> = None;
    let mut files = Vec::new();
    let mut total_upload_bytes: usize = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("failed to process multipart form data".to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == PAYLOAD_FIELD_NAME {
            if payload.is_some() {
                return Err(AppError::BadRequest(
                    "only one payload field is allowed in multipart request".to_string(),
                ));
            }

            let raw_payload = field
                .text()
                .await
                .map_err(|_| AppError::BadRequest("failed to read payload field".to_string()))?;

            let parsed_payload: CreateProjectDto = serde_json::from_str(&raw_payload)
                .map_err(|_| AppError::BadRequest("payload field must contain valid json".to_string()))?;

            parsed_payload
                .validate()
                .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

            payload = Some(parsed_payload);
            continue;
        }

        if field_name == FILES_FIELD_NAME {
            let original_name = field.file_name().unwrap_or("archivo").to_string();
            let mime_type = field.content_type().map(|value| value.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest("failed to read one of the uploaded files".to_string()))?;

            total_upload_bytes = total_upload_bytes.checked_add(bytes.len()).ok_or_else(|| {
                AppError::BadRequest("total uploaded file size exceeds the allowed limit".to_string())
            })?;

            if total_upload_bytes > MAX_TOTAL_UPLOAD_BYTES {
                return Err(AppError::BadRequest(
                    "total uploaded file size must not exceed 50mb".to_string(),
                ));
            }

            files.push(UploadedFile {
                original_name,
                mime_type,
                bytes: bytes.to_vec(),
            });
        }
    }

    let payload = payload.ok_or_else(|| {
        AppError::BadRequest("multipart request must include a payload field with project json".to_string())
    })?;

    Ok((payload, files))
}

async fn parse_project_create_request(
    state: &AppState,
    request: Request,
) -> Result<(CreateProjectDto, Vec<UploadedFile>), AppError> {
    if is_multipart_request(&request) {
        return parse_project_payload_from_multipart(state, request).await;
    }

    parse_project_payload_from_json(request).await
}

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
    let (payload, files) = parse_project_create_request(&state, request).await?;

    if files.is_empty() {
        let project = ProjectService::create(state.db.as_ref(), user.id, payload).await?;
        return Ok(Json(project));
    }

    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let project = match ProjectService::create_in_tx(&tx, user.id, payload).await {
        Ok(project) => project,
        Err(error) => {
            tx.rollback().await.map_err(AppError::Database)?;
            return Err(error);
        }
    };

    for file in files {
        let attachment = CreateAttachmentDto {
            project_id: Some(project.id),
            task_id: None,
            original_name: file.original_name,
            mime_type: file.mime_type,
        };

        if let Err(error) =
            AttachmentService::create_in_tx(&tx, state.storage.as_ref(), user.id, attachment, file.bytes).await
        {
            tx.rollback().await.map_err(AppError::Database)?;
            return Err(error);
        }
    }

    tx.commit().await.map_err(AppError::Database)?;

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
