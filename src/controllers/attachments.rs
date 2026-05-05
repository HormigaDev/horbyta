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
use jsonwebtoken::{DecodingKey, Validation, decode};
use object_store::{ObjectStore, ObjectStoreExt, path::Path as ObjectPath};
use sea_orm::{EntityTrait, TransactionTrait};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    constants::{CLIENT_AUTH_COOKIE_NAME, USER_AUTH_COOKIE_NAME},
    dtos::{
        attachment::{CreateAttachmentDto, UploadedFile, attachment_filter_schema},
        query::{Filter, FilterOperator, QueryPayload},
        user::UserDto,
    },
    entities::client_auth,
    error::AppError,
    middlewares::{
        auth::Claims,
        client_auth::ClientClaims,
        rate_limit::{UPLOADS_RATE_LIMIT, check_user_rate_limit},
        validator::ValidatedSearchQuery,
    },
    services::attachments::{AttachmentService, MAX_ATTACHMENT_FILE_BYTES},
    state::AppState,
};

const FILES_FIELD_NAME: &str = "files";

fn quote_filename_for_header(file_name: &str) -> String {
    file_name.replace('"', "")
}

async fn parse_uploaded_files(mut multipart: Multipart) -> Result<(Vec<UploadedFile>, i64), AppError> {
    let mut files = Vec::new();
    let mut total_upload_bytes: i64 = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("error processing multipart form".to_string()))?
    {
        if field.name().unwrap_or_default() != FILES_FIELD_NAME {
            continue;
        }

        let original_name = field.file_name().unwrap_or("archivo").to_string();
        let mime_type = field.content_type().map(|value| value.to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|_| AppError::BadRequest("could not read one of the files".to_string()))?;

        let file_size = i64::try_from(bytes.len())
            .map_err(|_| AppError::BadRequest("file exceeds the supported size".to_string()))?;

        if file_size > MAX_ATTACHMENT_FILE_BYTES {
            return Err(AppError::BadRequest("each file cannot exceed 50mb".to_string()));
        }

        total_upload_bytes = total_upload_bytes
            .checked_add(file_size)
            .ok_or_else(|| AppError::BadRequest("total file size exceeds the allowed limit".to_string()))?;

        files.push(UploadedFile {
            original_name,
            mime_type,
            bytes: bytes.to_vec(),
        });
    }

    if files.is_empty() {
        return Err(AppError::BadRequest(
            "you must send files in the files field".to_string(),
        ));
    }

    Ok((files, total_upload_bytes))
}

async fn cleanup_storage_objects(storage: &dyn ObjectStore, storage_keys: &[String]) {
    for storage_key in storage_keys {
        let location = match ObjectPath::parse(storage_key.as_str()) {
            Ok(location) => location,
            Err(error) => {
                tracing::warn!(storage_key = %storage_key, error = %error, "Invalid storage key during cleanup");
                continue;
            }
        };

        if let Err(error) = storage.delete(&location).await {
            tracing::warn!(storage_key = %storage_key, error = %error, "Attachment cleanup failed");
        }
    }
}

async fn upload_attachments_to_scope(
    state: &AppState,
    user_id: Uuid,
    project_id: Option<Uuid>,
    task_id: Option<Uuid>,
    multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    check_user_rate_limit(&state.rate_limit_uploads, user_id, UPLOADS_RATE_LIMIT)?;

    let (files, total_upload_bytes) = parse_uploaded_files(multipart).await?;
    AttachmentService::guard_upload_capacity(
        state.db.as_ref(),
        user_id,
        project_id,
        task_id,
        None,
        None,
        total_upload_bytes,
    )
    .await?;

    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let mut created_attachments = Vec::with_capacity(files.len());
    let mut created_storage_keys: Vec<String> = Vec::new();

    for file in files {
        let dto = CreateAttachmentDto {
            project_id,
            task_id,
            milestone_id: None,
            change_request_id: None,
            original_name: file.original_name,
            mime_type: file.mime_type,
        };

        match AttachmentService::create_in_tx(&tx, state.storage.as_ref(), user_id, dto, file.bytes).await {
            Ok(created) => {
                created_storage_keys.push(created.storage_key.clone());
                created_attachments.push(created);
            }
            Err(error) => {
                if let Err(rollback_error) = tx.rollback().await {
                    tracing::warn!(error = %rollback_error, "Attachment upload rollback failed");
                }

                cleanup_storage_objects(state.storage.as_ref(), &created_storage_keys).await;
                return Err(error);
            }
        }
    }

    if let Err(commit_error) = tx.commit().await {
        cleanup_storage_objects(state.storage.as_ref(), &created_storage_keys).await;
        return Err(AppError::Database(commit_error));
    }

    Ok(Json(json!({
        "items": created_attachments,
        "uploaded_bytes": total_upload_bytes,
        "uploaded_count": created_storage_keys.len()
    })))
}

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
    let mut payload = payload;
    payload.filters.push(Filter {
        column: "project_id".to_string(),
        operator: FilterOperator::Eq,
        value: Value::String(project_id.to_string()),
    });

    attachment_filter_schema().validate_payload(&payload)?;

    let (items, total_items) = AttachmentService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total_items
    })))
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
    let mut payload = payload;
    payload.filters.push(Filter {
        column: "task_id".to_string(),
        operator: FilterOperator::Eq,
        value: Value::String(task_id.to_string()),
    });

    attachment_filter_schema().validate_payload(&payload)?;

    let (items, total_items) = AttachmentService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total_items
    })))
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
    let user_id = resolve_attachment_viewer_user_id(&state, &jar).await?;
    let attachment = AttachmentService::get_by_id(state.db.as_ref(), user_id, id).await?;

    let storage_path = ObjectPath::parse(attachment.storage_key.as_str())
        .map_err(|error| AppError::Internal(format!("invalid storage key: {}", error)))?;

    let object = state
        .storage
        .get(&storage_path)
        .await
        .map_err(|error| AppError::NotFound(format!("file not found in storage: {}", error)))?;

    let bytes = object
        .bytes()
        .await
        .map_err(|error| AppError::Internal(format!("could not read file: {}", error)))?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str(&attachment.mime_type)?);

    let safe_name = quote_filename_for_header(&attachment.original_name);
    let disposition = format!("inline; filename=\"{}\"", safe_name);
    headers.insert(CONTENT_DISPOSITION, HeaderValue::from_str(&disposition)?);

    Ok((headers, bytes))
}

async fn resolve_attachment_viewer_user_id(state: &AppState, jar: &CookieJar) -> Result<Uuid, AppError> {
    if let Some(cookie) = jar.get(USER_AUTH_COOKIE_NAME) {
        let token = decode::<Claims>(
            cookie.value(),
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid user session".to_string()))?;

        return Ok(token.claims.sub);
    }

    if let Some(cookie) = jar.get(CLIENT_AUTH_COOKIE_NAME) {
        let token = decode::<ClientClaims>(
            cookie.value(),
            &DecodingKey::from_secret(state.config.jwt_secret_client.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid client session".to_string()))?;

        let client_auth = client_auth::Entity::find_by_id(token.claims.sub)
            .one(state.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Unauthorized("invalid client session".to_string()))?;

        return Ok(client_auth.user_id);
    }

    Err(AppError::Unauthorized("no session".to_string()))
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
    AttachmentService::delete(state.db.as_ref(), state.storage.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Adjunto eliminado" })))
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
) -> Result<Json<Value>, AppError> {
    upload_attachments_to_scope(&state, user.id, None, Some(task_id), multipart).await
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
) -> Result<Json<Value>, AppError> {
    upload_attachments_to_scope(&state, user.id, Some(project_id), None, multipart).await
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
