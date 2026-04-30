use axum::{
    Json, Router,
    body::to_bytes,
    extract::{FromRequest, Multipart, Path, Request, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::TransactionTrait;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        attachment::{CreateAttachmentDto, UploadedFile},
        query::QueryPayload,
        task::{CreateTaskDto, UpdateTaskDto, UpdateTaskStatusDto, task_filter_schema},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery, first_validation_error_message},
    services::{attachments::AttachmentService, tasks::TaskService},
    state::AppState,
};

const MAX_TOTAL_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
const MAX_JSON_BODY_BYTES: usize = 2 * 1024 * 1024;
const PAYLOAD_FIELD_NAME: &str = "payload";
const FILES_FIELD_NAME: &str = "files";

fn is_multipart_request(request: &Request) -> bool {
    request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|content_type| content_type.starts_with("multipart/form-data"))
        .unwrap_or(false)
}

async fn parse_task_payload_from_json(request: Request) -> Result<(CreateTaskDto, Vec<UploadedFile>), AppError> {
    let bytes = to_bytes(request.into_body(), MAX_JSON_BODY_BYTES)
        .await
        .map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

    let payload: CreateTaskDto =
        serde_json::from_slice(&bytes).map_err(|_| AppError::Unprocessable("invalid json payload".to_string()))?;

    payload
        .validate()
        .map_err(|errors| AppError::BadRequest(first_validation_error_message(&errors)))?;

    Ok((payload, Vec::new()))
}

async fn parse_task_payload_from_multipart(
    state: &AppState,
    request: Request,
) -> Result<(CreateTaskDto, Vec<UploadedFile>), AppError> {
    let mut multipart = Multipart::from_request(request, state)
        .await
        .map_err(|_| AppError::Unprocessable("invalid multipart payload".to_string()))?;

    let mut payload: Option<CreateTaskDto> = None;
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

            let parsed_payload: CreateTaskDto = serde_json::from_str(&raw_payload)
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
        AppError::BadRequest("multipart request must include a payload field with task json".to_string())
    })?;

    Ok((payload, files))
}

async fn parse_task_create_request(
    state: &AppState,
    request: Request,
) -> Result<(CreateTaskDto, Vec<UploadedFile>), AppError> {
    if is_multipart_request(&request) {
        return parse_task_payload_from_multipart(state, request).await;
    }

    parse_task_payload_from_json(request).await
}

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
    let (payload, files) = parse_task_create_request(&state, request).await?;

    if files.is_empty() {
        let task = TaskService::create(state.db.as_ref(), user.id, payload).await?;
        return Ok(Json(task));
    }

    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let task = match TaskService::create_in_tx(&tx, user.id, payload).await {
        Ok(task) => task,
        Err(error) => {
            tx.rollback().await.map_err(AppError::Database)?;
            return Err(error);
        }
    };

    for file in files {
        let attachment = CreateAttachmentDto {
            project_id: task.project_id,
            task_id: Some(task.id),
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

    Ok(Json(task))
}

async fn update_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let task = TaskService::update(state.db.as_ref(), user.id, id, payload).await?;
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
