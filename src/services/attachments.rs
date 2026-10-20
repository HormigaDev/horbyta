use axum::extract::Multipart;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};
use object_store::{ObjectStore, ObjectStoreExt, path::Path as ObjectPath};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Set, sea_query::Expr,
};
use serde_json::Value;
use std::env;
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{
    constants::{CLIENT_AUTH_COOKIE_NAME, USER_AUTH_COOKIE_NAME},
    dtos::{
        attachment::{
            AttachmentDeleteResponseDto, AttachmentDetailDto, AttachmentRawFileDto, AttachmentSearchField,
            AttachmentSearchResponseDto, AttachmentStorageUsageDto, AttachmentSummaryDto, AttachmentUploadResponseDto,
            AttachmentUploadScope, AttachmentViewer, CreateAttachmentDto, MarkdownAttachmentUploadResult,
            StorageBackedDbWrite, UploadedFile, attachment_filter_schema,
        },
        query::{Filter, FilterOperator, QueryPayload},
    },
    entities::{
        attachments::{self, Model as AttachmentModel},
        change_requests, client_auth, clients, milestones,
        prelude::{Attachments, ChangeRequests, Milestones, Projects, Tasks},
        projects, tasks,
    },
    error::AppError,
    middlewares::{auth::Claims, client_auth::ClientClaims},
    utils::{
        multipart::MarkdownUploadedFile,
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub const MAX_ATTACHMENT_FILE_BYTES: i64 = 50 * 1024 * 1024;
pub const MAX_ATTACHMENT_SCOPE_BYTES: i64 = 50 * 1024 * 1024;

const DEFAULT_ATTACHMENT_MIME: &str = "application/octet-stream";
const USER_STORAGE_LIMIT_GB_ENV: &str = "USER_STORAGE_LIMIT_GB";
const DEFAULT_USER_STORAGE_LIMIT_GB: u64 = 2;
const FILES_FIELD_NAME: &str = "files";

pub struct AttachmentService;

impl AttachmentService {
    pub async fn upload_files_to_scope(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        scope: AttachmentUploadScope,
        multipart: Multipart,
    ) -> Result<StorageBackedDbWrite<AttachmentUploadResponseDto>, AppError> {
        let (files, total_upload_bytes) = Self::parse_uploaded_files(multipart, false).await?;
        Self::guard_upload_capacity(
            db,
            user_id,
            scope.project_id,
            scope.task_id,
            scope.milestone_id,
            scope.change_request_id,
            total_upload_bytes,
        )
        .await?;

        let mut created_attachments = Vec::with_capacity(files.len());
        let mut created_storage_keys: Vec<String> = Vec::new();

        for file in files {
            let (dto, bytes) = scope.into_create_attachment_dto(file);

            match Self::create_in_tx(db, storage, user_id, dto, bytes).await {
                Ok(created) => {
                    created_storage_keys.push(created.storage_key.clone());
                    created_attachments.push(created);
                }
                Err(error) => {
                    Self::cleanup_storage_objects(storage, &created_storage_keys).await;
                    return Err(error);
                }
            }
        }

        Ok(StorageBackedDbWrite {
            value: AttachmentUploadResponseDto {
                uploaded_count: created_storage_keys.len(),
                uploaded_bytes: total_upload_bytes,
                items: created_attachments,
            },
            storage_keys: created_storage_keys,
        })
    }

    pub async fn upload_conversation_files(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        project_id: Uuid,
        multipart: Multipart,
    ) -> Result<StorageBackedDbWrite<AttachmentUploadResponseDto>, AppError> {
        let (files, total) = Self::parse_uploaded_files(multipart, true).await?;
        Self::guard_upload_capacity(db, user_id, Some(project_id), None, None, None, total).await?;
        let scope = AttachmentUploadScope::project(project_id);
        let mut items = Vec::new();
        let mut storage_keys = Vec::new();
        for file in files {
            let (dto, bytes) = scope.into_create_attachment_dto(file);
            match Self::create_in_tx(db, storage, user_id, dto, bytes).await {
                Ok(created) => {
                    storage_keys.push(created.storage_key.clone());
                    items.push(created);
                }
                Err(error) => {
                    Self::cleanup_storage_objects(storage, &storage_keys).await;
                    return Err(error);
                }
            }
        }
        Ok(StorageBackedDbWrite {
            value: AttachmentUploadResponseDto {
                uploaded_count: items.len(),
                uploaded_bytes: total,
                items,
            },
            storage_keys,
        })
    }

    pub async fn search_project(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        payload: QueryPayload,
    ) -> Result<AttachmentSearchResponseDto, AppError> {
        Self::search_scoped(db, user_id, payload, "project_id", project_id).await
    }

    pub async fn search_task(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        task_id: Uuid,
        payload: QueryPayload,
    ) -> Result<AttachmentSearchResponseDto, AppError> {
        Self::search_scoped(db, user_id, payload, "task_id", task_id).await
    }

    pub async fn get_raw_file_for_cookies(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        jwt_secret: &str,
        jwt_secret_client: &str,
        jar: &CookieJar,
        attachment_id: Uuid,
    ) -> Result<AttachmentRawFileDto, AppError> {
        let viewer = Self::resolve_attachment_viewer(db, jwt_secret, jwt_secret_client, jar).await?;
        let attachment = match viewer {
            AttachmentViewer::User { user_id } => Self::get_by_id(db, user_id, attachment_id).await?,
            AttachmentViewer::Client { user_id, client_id } => {
                Self::get_by_id_for_client(db, user_id, client_id, attachment_id).await?
            }
        };

        let storage_path = ObjectPath::parse(attachment.storage_key.as_str())
            .map_err(|error| AppError::Internal(format!("invalid storage key: {}", error)))?;

        let object = storage.get(&storage_path).await.map_err(|error| {
            tracing::warn!("attachment missing in storage: {error}");
            AppError::NotFound(codes::ATTACHMENT_FILE_NOT_IN_STORAGE.into())
        })?;

        let bytes = object
            .bytes()
            .await
            .map_err(|error| AppError::Internal(format!("could not read file: {}", error)))?;

        let preserved=db.query_one(sea_orm::Statement::from_sql_and_values(sea_orm::DbBackend::Postgres,
            "SELECT a->>'sha256' AS hash FROM scope_revisions r CROSS JOIN jsonb_array_elements(r.attachment_snapshot) a WHERE a->>'id'=$1 AND a->>'sha256' IS NOT NULL AND r.published ORDER BY r.created_at DESC LIMIT 1",[attachment_id.to_string().into()])).await?;
        if let Some(row) = preserved {
            use sha2::{Digest, Sha256};
            if hex::encode(Sha256::digest(&bytes)) != row.try_get::<String>("", "hash")? {
                return Err(AppError::Conflict(codes::ATTACHMENT_PUBLISHED_MISMATCH.into()));
            }
        }
        Ok(AttachmentRawFileDto {
            mime_type: attachment.mime_type.clone(),
            content_disposition: Self::build_content_disposition(&attachment.original_name, &attachment.mime_type),
            bytes: bytes.to_vec(),
        })
    }

    pub async fn delete_with_response(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<AttachmentDeleteResponseDto, AppError> {
        Self::delete(db, storage, user_id, attachment_id).await?;
        Ok(AttachmentDeleteResponseDto {
            message: "Adjunto eliminado".to_string(),
        })
    }

    pub async fn create_markdown_files_in_tx(
        tx: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<Vec<(String, String)>, AppError> {
        Ok(Self::create_markdown_files_with_tracking_in_tx(
            tx,
            storage,
            user_id,
            project_id,
            task_id,
            milestone_id,
            change_request_id,
            files,
        )
        .await?
        .replacements)
    }

    pub async fn create_markdown_files_with_tracking_in_tx(
        tx: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<MarkdownAttachmentUploadResult, AppError> {
        let mut replacements = Vec::with_capacity(files.len());
        let mut storage_keys = Vec::with_capacity(files.len());

        for file in files {
            let local_id = file.local_id;
            let attachment = CreateAttachmentDto {
                project_id,
                task_id,
                milestone_id,
                change_request_id,
                original_name: file.original_name,
                mime_type: file.mime_type,
            };

            let created = match Self::create_in_tx(tx, storage, user_id, attachment, file.bytes).await {
                Ok(created) => created,
                Err(error) => {
                    Self::cleanup_storage_objects(storage, &storage_keys).await;
                    return Err(error);
                }
            };
            storage_keys.push(created.storage_key.clone());
            replacements.push((local_id, format!("/api/v1/attachments/{}/raw", created.id)));
        }

        Ok(MarkdownAttachmentUploadResult {
            replacements,
            storage_keys,
        })
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateAttachmentDto,
        file_bytes: Vec<u8>,
    ) -> Result<AttachmentDetailDto, AppError> {
        let model = Self::create_with_connection(db, storage, user_id, dto, file_bytes).await?;
        Ok(Self::to_detail_dto(model))
    }

    pub async fn create_in_tx(
        tx: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateAttachmentDto,
        file_bytes: Vec<u8>,
    ) -> Result<AttachmentDetailDto, AppError> {
        let model = Self::create_with_connection(tx, storage, user_id, dto, file_bytes).await?;
        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_user_storage_usage(
        db: &impl ConnectionTrait,
        user_id: Uuid,
    ) -> Result<AttachmentStorageUsageDto, AppError> {
        let used_bytes = Self::sum_user_size_bytes(db, user_id).await?;
        let limit_bytes = Self::resolve_user_storage_limit_bytes()?;

        Ok(AttachmentStorageUsageDto {
            used_bytes,
            limit_bytes,
            remaining_bytes: limit_bytes.saturating_sub(used_bytes),
        })
    }

    pub async fn guard_upload_capacity(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
        upload_bytes: i64,
    ) -> Result<(), AppError> {
        if upload_bytes <= 0 {
            return Err(AppError::BadRequest(codes::ATTACHMENT_FILE_REQUIRED.into()));
        }

        let scope_name = if task_id.is_some() {
            "task"
        } else if milestone_id.is_some() {
            "milestone"
        } else if change_request_id.is_some() {
            "change request"
        } else if project_id.is_some() {
            "project"
        } else {
            return Err(AppError::BadRequest(codes::ATTACHMENT_RESOURCE_REQUIRED.into()));
        };

        if upload_bytes > MAX_ATTACHMENT_SCOPE_BYTES {
            return Err(AppError::BadRequest(
                codes::ATTACHMENT_RESOURCE_SIZE_EXCEEDED.with([scope_name.to_string()]),
            ));
        }

        let current_scope_usage =
            Self::sum_scope_size_bytes(db, user_id, project_id, task_id, milestone_id, change_request_id).await?;
        let next_scope_usage = i128::from(current_scope_usage) + i128::from(upload_bytes);
        if next_scope_usage > i128::from(MAX_ATTACHMENT_SCOPE_BYTES) {
            return Err(AppError::BadRequest(
                codes::ATTACHMENT_RESOURCE_LIMIT_REACHED.with([scope_name.to_string()]),
            ));
        }

        let user_limit_bytes = Self::resolve_user_storage_limit_bytes()?;
        let current_user_usage = Self::sum_user_size_bytes(db, user_id).await?;
        let next_user_usage = i128::from(current_user_usage) + i128::from(upload_bytes);
        if next_user_usage > i128::from(user_limit_bytes) {
            return Err(AppError::BadRequest(codes::ATTACHMENT_STORAGE_LIMIT_REACHED.into()));
        }

        Ok(())
    }

    async fn create_with_connection(
        conn: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateAttachmentDto,
        file_bytes: Vec<u8>,
    ) -> Result<AttachmentModel, AppError> {
        let (project_id, task_id, milestone_id, change_request_id) = Self::resolve_targets(
            conn,
            user_id,
            dto.project_id,
            dto.task_id,
            dto.milestone_id,
            dto.change_request_id,
        )
        .await?;

        let original_name = dto.original_name.trim().to_string();
        if original_name.is_empty() {
            return Err(AppError::BadRequest(codes::ATTACHMENT_NAME_EMPTY.into()));
        }

        if file_bytes.is_empty() {
            return Err(AppError::BadRequest(codes::ATTACHMENT_FILE_EMPTY.into()));
        }

        let size_bytes = i64::try_from(file_bytes.len())
            .map_err(|_| AppError::BadRequest(codes::ATTACHMENT_FILE_EXCEEDS_SIZE.into()))?;

        if size_bytes > MAX_ATTACHMENT_FILE_BYTES {
            return Err(AppError::BadRequest(codes::ATTACHMENT_FILE_TOO_LARGE.into()));
        }

        Self::guard_upload_capacity(
            conn,
            user_id,
            project_id,
            task_id,
            milestone_id,
            change_request_id,
            size_bytes,
        )
        .await?;

        let mime_type = dto
            .mime_type
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_ATTACHMENT_MIME.to_string());

        let storage_key = Self::build_storage_key(
            user_id,
            project_id,
            task_id,
            milestone_id,
            change_request_id,
            &original_name,
        );
        let location = ObjectPath::parse(storage_key.as_str())
            .map_err(|error| AppError::Internal(format!("invalid storage key: {}", error)))?;

        storage
            .put(&location, file_bytes.into())
            .await
            .map_err(|error| AppError::Internal(format!("could not save file to storage: {}", error)))?;

        let inserted = attachments::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(project_id),
            task_id: Set(task_id),
            milestone_id: Set(milestone_id),
            change_request_id: Set(change_request_id),
            original_name: Set(original_name),
            storage_key: Set(storage_key.clone()),
            mime_type: Set(mime_type),
            size_bytes: Set(size_bytes),
            ..Default::default()
        }
        .insert(conn)
        .await;

        match inserted {
            Ok(model) => Ok(model),
            Err(error) => {
                if let Err(storage_error) = storage.delete(&location).await {
                    tracing::warn!(
                        storage_key = %storage_key,
                        error = %storage_error,
                        "Attachment rollback failed after DB insertion error"
                    );
                }

                return Err(AppError::Database(error));
            }
        }
    }

    pub async fn get_by_id(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<AttachmentDetailDto, AppError> {
        let model = Self::find_by_id(db, user_id, attachment_id).await?;
        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_by_id_for_client(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<AttachmentDetailDto, AppError> {
        let model = Self::find_by_id(db, user_id, attachment_id).await?;
        Self::ensure_client_can_access_attachment(db, user_id, client_id, &model).await?;
        Ok(Self::to_detail_dto(model))
    }

    pub async fn delete(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<(), AppError> {
        let model = Self::find_by_id(db, user_id, attachment_id).await?;

        let location = ObjectPath::parse(model.storage_key.as_str())
            .map_err(|error| AppError::Internal(format!("invalid storage key: {}", error)))?;

        attachments::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::from)?;

        if let Err(error) = storage.delete(&location).await {
            tracing::warn!(
                attachment_id = %model.id,
                storage_key = %model.storage_key,
                error = %error,
                "Failed to remove attachment from storage, continuing with DB delete"
            );
        }

        Ok(())
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<AttachmentSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(attachments::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<AttachmentSearchField>()
                .ok()
                .map(AttachmentSearchField::filter_field)
        })?;

        let query = Attachments::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            AttachmentSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<AttachmentSearchField>()
                    .ok()
                    .map(AttachmentSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await.map_err(AppError::Database)?;
        let models = apply_pagination(
            query
                .select_only()
                .column(attachments::Column::Id)
                .column(attachments::Column::ProjectId)
                .column(attachments::Column::TaskId)
                .column(attachments::Column::MilestoneId)
                .column(attachments::Column::ChangeRequestId)
                .column(attachments::Column::OriginalName)
                .column(attachments::Column::MimeType)
                .column(attachments::Column::SizeBytes)
                .column(attachments::Column::CreatedAt)
                .column(attachments::Column::UpdatedAt),
            &payload,
        )
        .into_model::<AttachmentSummaryDto>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

        Ok((models, total_items))
    }

    async fn search_scoped(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        mut payload: QueryPayload,
        scope_column: &str,
        scope_id: Uuid,
    ) -> Result<AttachmentSearchResponseDto, AppError> {
        payload.filters.push(Filter {
            column: scope_column.to_string(),
            operator: FilterOperator::Eq,
            value: Value::String(scope_id.to_string()),
        });

        attachment_filter_schema().validate_payload(&payload)?;
        let (items, total) = Self::search(db, user_id, payload.clone()).await?;

        Ok(AttachmentSearchResponseDto { payload, items, total })
    }

    async fn parse_uploaded_files(
        mut multipart: Multipart,
        conversation: bool,
    ) -> Result<(Vec<UploadedFile>, i64), AppError> {
        let mut files = Vec::new();
        let mut total_upload_bytes: i64 = 0;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| AppError::BadRequest(codes::ATTACHMENT_MULTIPART_INVALID.into()))?
        {
            if field.name().unwrap_or_default() != FILES_FIELD_NAME {
                continue;
            }

            let original_name = field.file_name().unwrap_or("archivo").to_string();
            let mime_type = field.content_type().map(|value| value.to_string());
            if conversation {
                let extension = original_name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                if mime_type
                    .as_deref()
                    .is_some_and(|mime| mime.to_ascii_lowercase().starts_with("audio/"))
                    || [
                        "mp3", "wav", "ogg", "oga", "opus", "m4a", "aac", "flac", "aiff", "aif", "amr", "wma",
                    ]
                    .contains(&extension.as_str())
                {
                    return Err(AppError::BadRequest(codes::ATTACHMENT_AUDIO_NOT_SUPPORTED.into()));
                }
                if files.len() >= 10 {
                    return Err(AppError::BadRequest(codes::CONVERSATION_TOO_MANY_ATTACHMENTS.into()));
                }
            }
            let bytes = field
                .bytes()
                .await
                .map_err(|_| AppError::BadRequest(codes::ATTACHMENT_READ_FAILED.into()))?;

            let file_size = i64::try_from(bytes.len())
                .map_err(|_| AppError::BadRequest(codes::ATTACHMENT_FILE_EXCEEDS_SIZE.into()))?;

            if file_size > MAX_ATTACHMENT_FILE_BYTES {
                return Err(AppError::BadRequest(codes::ATTACHMENT_FILE_TOO_LARGE.into()));
            }

            total_upload_bytes = total_upload_bytes
                .checked_add(file_size)
                .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_TOTAL_SIZE_EXCEEDED.into()))?;

            if conversation && total_upload_bytes > MAX_ATTACHMENT_SCOPE_BYTES {
                return Err(AppError::BadRequest(codes::ATTACHMENT_TOTAL_SIZE_EXCEEDED.into()));
            }
            files.push(UploadedFile {
                original_name,
                mime_type,
                bytes: bytes.to_vec(),
            });
        }

        if files.is_empty() {
            return Err(AppError::BadRequest(codes::ATTACHMENT_FILES_FIELD_REQUIRED.into()));
        }

        Ok((files, total_upload_bytes))
    }

    pub async fn cleanup_storage_objects(storage: &dyn ObjectStore, storage_keys: &[String]) {
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

    async fn resolve_attachment_viewer(
        db: &impl ConnectionTrait,
        jwt_secret: &str,
        jwt_secret_client: &str,
        jar: &CookieJar,
    ) -> Result<AttachmentViewer, AppError> {
        if let Some(cookie) = jar.get(USER_AUTH_COOKIE_NAME) {
            let token = decode::<Claims>(
                cookie.value(),
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AppError::Unauthorized(codes::ATTACHMENT_USER_SESSION_INVALID.into()))?;

            return Ok(AttachmentViewer::User {
                user_id: token.claims.sub,
            });
        }

        if let Some(cookie) = jar.get(CLIENT_AUTH_COOKIE_NAME) {
            let token = decode::<ClientClaims>(
                cookie.value(),
                &DecodingKey::from_secret(jwt_secret_client.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AppError::Unauthorized(codes::ATTACHMENT_CLIENT_SESSION_INVALID.into()))?;

            let client_auth = client_auth::Entity::find_by_id(token.claims.sub)
                .one(db)
                .await
                .map_err(AppError::Database)?
                .ok_or_else(|| AppError::Unauthorized(codes::ATTACHMENT_CLIENT_SESSION_INVALID.into()))?;

            let client_profile = client_auth
                .find_related(clients::Entity)
                .one(db)
                .await
                .map_err(AppError::Database)?
                .ok_or_else(|| AppError::Unauthorized(codes::CLIENT_NOT_FOUND.into()))?;

            if client_profile.status != "active" {
                return Err(AppError::Forbidden(codes::AUTH_CLIENT_ACCESS_INACTIVE.into()));
            }

            return Ok(AttachmentViewer::Client {
                user_id: client_auth.user_id,
                client_id: client_auth.client_id,
            });
        }

        Err(AppError::Unauthorized(codes::AUTH_NO_SESSION.into()))
    }

    fn build_content_disposition(file_name: &str, mime_type: &str) -> String {
        let safe_name = Self::quote_filename_for_header(file_name);
        let disposition_type = if Self::is_inline_safe_mime_type(mime_type) {
            "inline"
        } else {
            "attachment"
        };

        format!("{disposition_type}; filename=\"{safe_name}\"")
    }

    fn quote_filename_for_header(file_name: &str) -> String {
        file_name
            .chars()
            .filter(|ch| !matches!(ch, '"' | '\\' | '\r' | '\n') && !ch.is_control())
            .collect()
    }

    fn is_inline_safe_mime_type(mime_type: &str) -> bool {
        let normalized = mime_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        matches!(
            normalized.as_str(),
            "application/pdf"
                | "image/apng"
                | "image/avif"
                | "image/gif"
                | "image/jpeg"
                | "image/png"
                | "image/webp"
                | "text/plain"
        )
    }

    fn resolve_user_storage_limit_bytes() -> Result<i64, AppError> {
        let limit_gb = env::var(USER_STORAGE_LIMIT_GB_ENV)
            .unwrap_or_else(|_| DEFAULT_USER_STORAGE_LIMIT_GB.to_string())
            .parse::<u64>()
            .map_err(|_| {
                AppError::Internal(format!(
                    "invalid configuration: {} must be a positive integer",
                    USER_STORAGE_LIMIT_GB_ENV
                ))
            })?;

        let bytes = u128::from(limit_gb) * 1024_u128 * 1024_u128 * 1024_u128;
        if bytes > i64::MAX as u128 {
            return Err(AppError::Internal(
                "configured storage limit exceeds the supported maximum".to_string(),
            ));
        }

        Ok(bytes as i64)
    }

    async fn sum_scope_size_bytes(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
    ) -> Result<i64, AppError> {
        let condition = if let Some(tid) = task_id {
            Condition::all()
                .add(attachments::Column::UserId.eq(user_id))
                .add(attachments::Column::TaskId.eq(tid))
        } else if let Some(mid) = milestone_id {
            Condition::all()
                .add(attachments::Column::UserId.eq(user_id))
                .add(attachments::Column::MilestoneId.eq(mid))
        } else if let Some(cid) = change_request_id {
            Condition::all()
                .add(attachments::Column::UserId.eq(user_id))
                .add(attachments::Column::ChangeRequestId.eq(cid))
        } else if let Some(pid) = project_id {
            Condition::all()
                .add(attachments::Column::UserId.eq(user_id))
                .add(attachments::Column::ProjectId.eq(pid))
                .add(attachments::Column::TaskId.is_null())
                .add(attachments::Column::MilestoneId.is_null())
                .add(attachments::Column::ChangeRequestId.is_null())
        } else {
            return Err(AppError::BadRequest(codes::ATTACHMENT_RESOURCE_REQUIRED.into()));
        };

        Self::sum_size_bytes(db, condition).await
    }

    async fn sum_user_size_bytes(db: &impl ConnectionTrait, user_id: Uuid) -> Result<i64, AppError> {
        let condition = Condition::all().add(attachments::Column::UserId.eq(user_id));
        Self::sum_size_bytes(db, condition).await
    }

    async fn sum_size_bytes(db: &impl ConnectionTrait, condition: Condition) -> Result<i64, AppError> {
        let total_size = Attachments::find()
            .filter(condition)
            .select_only()
            .column_as(Expr::cust("COALESCE(SUM(size_bytes), 0)::BIGINT"), "total_size")
            .into_tuple::<i64>()
            .one(db)
            .await
            .map_err(AppError::Database)?
            .unwrap_or(0);

        Ok(total_size)
    }

    async fn resolve_targets(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
    ) -> Result<(Option<Uuid>, Option<Uuid>, Option<Uuid>, Option<Uuid>), AppError> {
        if project_id.is_none() && task_id.is_none() && milestone_id.is_none() && change_request_id.is_none() {
            return Err(AppError::BadRequest(codes::ATTACHMENT_RESOURCE_REQUIRED.into()));
        }

        let owned_project_id = if let Some(pid) = project_id {
            let project = Projects::find()
                .filter(projects::Column::Id.eq(pid))
                .filter(projects::Column::UserId.eq(user_id))
                .one(db)
                .await
                .map_err(AppError::Database)?;

            if project.is_none() {
                return Err(AppError::BadRequest(codes::ATTACHMENT_PROJECT_FORBIDDEN.into()));
            }

            Some(pid)
        } else {
            None
        };

        let owned_task = if let Some(tid) = task_id {
            let task = Tasks::find()
                .filter(tasks::Column::Id.eq(tid))
                .filter(tasks::Column::UserId.eq(user_id))
                .one(db)
                .await
                .map_err(AppError::Database)?;

            let task = task.ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_TASK_FORBIDDEN.into()))?;

            if let Some(project_id_from_dto) = owned_project_id {
                if task.project_id != Some(project_id_from_dto) {
                    return Err(AppError::BadRequest(codes::TASK_NOT_IN_PROJECT.into()));
                }
            }

            Some(task)
        } else {
            None
        };

        let resolved_project_id = owned_project_id.or_else(|| owned_task.as_ref().and_then(|task| task.project_id));
        let resolved_task_id = owned_task.as_ref().map(|task| task.id);

        let owned_milestone = if let Some(mid) = milestone_id {
            let milestone = Milestones::find()
                .filter(milestones::Column::Id.eq(mid))
                .find_also_related(Projects)
                .one(db)
                .await
                .map_err(AppError::Database)?;

            match milestone {
                Some((milestone, Some(project))) if project.user_id == user_id => Some(milestone),
                _ => return Err(AppError::BadRequest(codes::ATTACHMENT_MILESTONE_FORBIDDEN.into())),
            }
        } else {
            None
        };

        let owned_change_request = if let Some(cid) = change_request_id {
            let change_request = ChangeRequests::find()
                .filter(change_requests::Column::Id.eq(cid))
                .filter(change_requests::Column::UserId.eq(user_id))
                .one(db)
                .await
                .map_err(AppError::Database)?
                .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_CHANGE_REQUEST_FORBIDDEN.into()))?;

            Some(change_request)
        } else {
            None
        };

        let resolved_project_id = resolved_project_id
            .or_else(|| owned_milestone.as_ref().map(|milestone| milestone.project_id))
            .or_else(|| {
                owned_change_request
                    .as_ref()
                    .map(|change_request| change_request.project_id)
            });
        let resolved_milestone_id = owned_milestone.as_ref().map(|milestone| milestone.id);
        let resolved_change_request_id = owned_change_request.as_ref().map(|change_request| change_request.id);

        Ok((
            resolved_project_id,
            resolved_task_id,
            resolved_milestone_id,
            resolved_change_request_id,
        ))
    }

    async fn find_by_id(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<AttachmentModel, AppError> {
        let attachment = Attachments::find()
            .filter(attachments::Column::Id.eq(attachment_id))
            .filter(attachments::Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()))?;

        Ok(attachment)
    }

    async fn ensure_client_can_access_attachment(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        attachment: &AttachmentModel,
    ) -> Result<(), AppError> {
        let mut checked_scope = false;

        if let Some(project_id) = attachment.project_id {
            checked_scope = true;
            if !Self::client_can_access_project(db, user_id, client_id, project_id).await? {
                return Err(AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()));
            }
        }

        if let Some(task_id) = attachment.task_id {
            checked_scope = true;
            if !Self::client_can_access_task(db, user_id, client_id, task_id).await? {
                return Err(AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()));
            }
        }

        if let Some(milestone_id) = attachment.milestone_id {
            checked_scope = true;
            if !Self::client_can_access_milestone(db, user_id, client_id, milestone_id).await? {
                return Err(AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()));
            }
        }

        if let Some(change_request_id) = attachment.change_request_id {
            checked_scope = true;
            if !Self::client_can_access_change_request(db, user_id, client_id, change_request_id).await? {
                return Err(AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()));
            }
        }

        if checked_scope {
            Ok(())
        } else {
            Err(AppError::NotFound(codes::ATTACHMENT_NOT_FOUND.into()))
        }
    }

    async fn client_can_access_project(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .is_some();

        Ok(exists)
    }

    async fn client_can_access_task(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        task_id: Uuid,
    ) -> Result<bool, AppError> {
        let task_with_project = Tasks::find()
            .filter(tasks::Column::Id.eq(task_id))
            .filter(tasks::Column::UserId.eq(user_id))
            .find_also_related(Projects)
            .one(db)
            .await
            .map_err(AppError::Database)?;

        Ok(matches!(
            task_with_project,
            Some((_task, Some(project))) if project.user_id == user_id && project.client_id == Some(client_id)
        ))
    }

    async fn client_can_access_milestone(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        milestone_id: Uuid,
    ) -> Result<bool, AppError> {
        let milestone_with_project = Milestones::find()
            .filter(milestones::Column::Id.eq(milestone_id))
            .find_also_related(Projects)
            .one(db)
            .await
            .map_err(AppError::Database)?;

        Ok(matches!(
            milestone_with_project,
            Some((_milestone, Some(project))) if project.user_id == user_id && project.client_id == Some(client_id)
        ))
    }

    async fn client_can_access_change_request(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        change_request_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists = ChangeRequests::find()
            .filter(change_requests::Column::Id.eq(change_request_id))
            .filter(change_requests::Column::UserId.eq(user_id))
            .filter(change_requests::Column::ClientId.eq(client_id))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .is_some();

        Ok(exists)
    }

    fn build_storage_key(
        user_id: Uuid,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        milestone_id: Option<Uuid>,
        change_request_id: Option<Uuid>,
        original_name: &str,
    ) -> String {
        let mut segments = vec!["attachments".to_string(), user_id.to_string()];

        if let Some(pid) = project_id {
            segments.push("projects".to_string());
            segments.push(pid.to_string());
        }

        if let Some(tid) = task_id {
            segments.push("tasks".to_string());
            segments.push(tid.to_string());
        }

        if let Some(mid) = milestone_id {
            segments.push("milestones".to_string());
            segments.push(mid.to_string());
        }

        if let Some(cid) = change_request_id {
            segments.push("change-requests".to_string());
            segments.push(cid.to_string());
        }

        let extension = std::path::Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.trim().to_ascii_lowercase())
            .filter(|ext| !ext.is_empty() && ext.len() <= 16 && ext.chars().all(|c| c.is_ascii_alphanumeric()));

        let file_name = match extension {
            Some(ext) => format!("{}.{}", Uuid::now_v7(), ext),
            None => Uuid::now_v7().to_string(),
        };

        segments.push(file_name);

        segments.join("/")
    }

    fn to_detail_dto(model: AttachmentModel) -> AttachmentDetailDto {
        AttachmentDetailDto {
            id: model.id,
            user_id: model.user_id,
            project_id: model.project_id,
            task_id: model.task_id,
            milestone_id: model.milestone_id,
            change_request_id: model.change_request_id,
            original_name: model.original_name,
            storage_key: model.storage_key,
            mime_type: model.mime_type,
            size_bytes: model.size_bytes,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
