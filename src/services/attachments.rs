use object_store::{ObjectStore, ObjectStoreExt, path::Path as ObjectPath};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set, TransactionTrait, sea_query::Expr,
};
use std::env;
use uuid::Uuid;

use crate::{
    dtos::{
        attachment::{
            AttachmentDetailDto, AttachmentSearchField, AttachmentStorageUsageDto, AttachmentSummaryDto,
            CreateAttachmentDto,
        },
        query::QueryPayload,
    },
    entities::{
        attachments::{self, Model as AttachmentModel},
        change_requests, milestones,
        prelude::{Attachments, ChangeRequests, Milestones, Projects, Tasks},
        projects, tasks,
    },
    error::AppError,
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

pub struct AttachmentService;

impl AttachmentService {
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
        let mut replacements = Vec::with_capacity(files.len());

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

            let created = Self::create_in_tx(tx, storage, user_id, attachment, file.bytes).await?;
            replacements.push((local_id, format!("/api/v1/attachments/{}/raw", created.id)));
        }

        Ok(replacements)
    }

    pub async fn create(
        db: &(impl ConnectionTrait + TransactionTrait),
        active_tx: Option<&impl ConnectionTrait>,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateAttachmentDto,
        file_bytes: Vec<u8>,
    ) -> Result<AttachmentDetailDto, AppError> {
        if let Some(tx) = active_tx {
            let model = Self::create_with_connection(tx, storage, user_id, dto, file_bytes).await?;
            return Ok(Self::to_detail_dto(model));
        }

        let tx = db.begin().await.map_err(AppError::Database)?;
        let model = Self::create_with_connection(&tx, storage, user_id, dto, file_bytes).await?;
        tx.commit().await.map_err(AppError::Database)?;

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
            return Err(AppError::BadRequest(
                "you must send at least one valid file".to_string(),
            ));
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
            return Err(AppError::BadRequest(
                "file must be associated with a markdown resource".to_string(),
            ));
        };

        if upload_bytes > MAX_ATTACHMENT_SCOPE_BYTES {
            return Err(AppError::BadRequest(format!(
                "total file size cannot exceed 50mb per {}",
                scope_name
            )));
        }

        let current_scope_usage =
            Self::sum_scope_size_bytes(db, user_id, project_id, task_id, milestone_id, change_request_id).await?;
        let next_scope_usage = i128::from(current_scope_usage) + i128::from(upload_bytes);
        if next_scope_usage > i128::from(MAX_ATTACHMENT_SCOPE_BYTES) {
            return Err(AppError::BadRequest(format!(
                "this {} has reached its 50mb attachment limit",
                scope_name
            )));
        }

        let user_limit_bytes = Self::resolve_user_storage_limit_bytes()?;
        let current_user_usage = Self::sum_user_size_bytes(db, user_id).await?;
        let next_user_usage = i128::from(current_user_usage) + i128::from(upload_bytes);
        if next_user_usage > i128::from(user_limit_bytes) {
            return Err(AppError::BadRequest(
                "you have reached your total storage limit for attachments".to_string(),
            ));
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
            return Err(AppError::BadRequest("original file name cannot be empty".to_string()));
        }

        if file_bytes.is_empty() {
            return Err(AppError::BadRequest("file cannot be empty".to_string()));
        }

        let size_bytes = i64::try_from(file_bytes.len())
            .map_err(|_| AppError::BadRequest("file exceeds the supported size".to_string()))?;

        if size_bytes > MAX_ATTACHMENT_FILE_BYTES {
            return Err(AppError::BadRequest("each file cannot exceed 50mb".to_string()));
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

    pub async fn delete(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<(), AppError> {
        let model = Self::find_by_id(db, user_id, attachment_id).await?;

        let location = ObjectPath::parse(model.storage_key.as_str())
            .map_err(|error| AppError::Internal(format!("invalid storage key: {}", error)))?;

        if let Err(error) = storage.delete(&location).await {
            tracing::warn!(
                attachment_id = %model.id,
                storage_key = %model.storage_key,
                error = %error,
                "Failed to remove attachment from storage, continuing with DB delete"
            );
        }

        attachments::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::Database)?;

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
            return Err(AppError::BadRequest(
                "file must be associated with a markdown resource".to_string(),
            ));
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
            return Err(AppError::BadRequest(
                "file must be associated with a markdown resource".to_string(),
            ));
        }

        let owned_project_id = if let Some(pid) = project_id {
            let project = Projects::find()
                .filter(projects::Column::Id.eq(pid))
                .filter(projects::Column::UserId.eq(user_id))
                .one(db)
                .await
                .map_err(AppError::Database)?;

            if project.is_none() {
                return Err(AppError::BadRequest("invalid project or no permissions".to_string()));
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

            let task = task.ok_or_else(|| AppError::BadRequest("invalid task or no permissions".to_string()))?;

            if let Some(project_id_from_dto) = owned_project_id {
                if task.project_id != Some(project_id_from_dto) {
                    return Err(AppError::BadRequest(
                        "task does not belong to the specified project".to_string(),
                    ));
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
                _ => return Err(AppError::BadRequest("invalid milestone or no permissions".to_string())),
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
                .ok_or_else(|| AppError::BadRequest("invalid change request or no permissions".to_string()))?;

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
            .ok_or_else(|| AppError::NotFound("attachment not found".to_string()))?;

        Ok(attachment)
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
