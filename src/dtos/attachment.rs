use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema, QueryPayload},
    entities::attachments,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum AttachmentSearchField {
    Id,
    ProjectId,
    TaskId,
    MilestoneId,
    ChangeRequestId,
    OriginalName,
    MimeType,
    SizeBytes,
    CreatedAt,
    UpdatedAt,
}

impl AttachmentSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::ProjectId => "project_id",
            Self::TaskId => "task_id",
            Self::MilestoneId => "milestone_id",
            Self::ChangeRequestId => "change_request_id",
            Self::OriginalName => "original_name",
            Self::MimeType => "mime_type",
            Self::SizeBytes => "size_bytes",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::ProjectId => ColumnType::Text,
            Self::TaskId => ColumnType::Text,
            Self::MilestoneId => ColumnType::Text,
            Self::ChangeRequestId => ColumnType::Text,
            Self::OriginalName => ColumnType::Text,
            Self::MimeType => ColumnType::Text,
            Self::SizeBytes => ColumnType::Number,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<attachments::Column> {
        match self {
            Self::Id => FilterField::Uuid(attachments::Column::Id),
            Self::ProjectId => FilterField::Uuid(attachments::Column::ProjectId),
            Self::TaskId => FilterField::Uuid(attachments::Column::TaskId),
            Self::MilestoneId => FilterField::Uuid(attachments::Column::MilestoneId),
            Self::ChangeRequestId => FilterField::Uuid(attachments::Column::ChangeRequestId),
            Self::OriginalName => FilterField::Text(attachments::Column::OriginalName),
            Self::MimeType => FilterField::Text(attachments::Column::MimeType),
            Self::SizeBytes => FilterField::NumberI64(attachments::Column::SizeBytes),
            Self::CreatedAt => FilterField::DateTime(attachments::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(attachments::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> attachments::Column {
        match self {
            Self::Id => attachments::Column::Id,
            Self::ProjectId => attachments::Column::ProjectId,
            Self::TaskId => attachments::Column::TaskId,
            Self::MilestoneId => attachments::Column::MilestoneId,
            Self::ChangeRequestId => attachments::Column::ChangeRequestId,
            Self::OriginalName => attachments::Column::OriginalName,
            Self::MimeType => attachments::Column::MimeType,
            Self::SizeBytes => attachments::Column::SizeBytes,
            Self::CreatedAt => attachments::Column::CreatedAt,
            Self::UpdatedAt => attachments::Column::UpdatedAt,
        }
    }
}

pub fn attachment_filter_schema() -> FilterSchema {
    AttachmentSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone, ToSchema)]
pub struct CreateAttachmentDto {
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub change_request_id: Option<Uuid>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "original file name must be between 1 and 255 characters"
    ))]
    pub original_name: String,

    #[validate(length(min = 1, max = 100, message = "mime type must be between 1 and 100 characters"))]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct AttachmentSummaryDto {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub change_request_id: Option<Uuid>,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentDetailDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub change_request_id: Option<Uuid>,
    pub original_name: String,
    pub storage_key: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentStorageUsageDto {
    pub used_bytes: i64,
    pub limit_bytes: i64,
    pub remaining_bytes: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentSearchResponseDto {
    pub payload: QueryPayload,
    pub items: Vec<AttachmentSummaryDto>,
    pub total: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentUploadResponseDto {
    pub items: Vec<AttachmentDetailDto>,
    pub uploaded_bytes: i64,
    pub uploaded_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct AttachmentUploadScope {
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub change_request_id: Option<Uuid>,
}

impl AttachmentUploadScope {
    pub const fn project(project_id: Uuid) -> Self {
        Self {
            project_id: Some(project_id),
            task_id: None,
            milestone_id: None,
            change_request_id: None,
        }
    }

    pub const fn task(task_id: Uuid) -> Self {
        Self {
            project_id: None,
            task_id: Some(task_id),
            milestone_id: None,
            change_request_id: None,
        }
    }

    pub fn into_create_attachment_dto(self, file: UploadedFile) -> (CreateAttachmentDto, Vec<u8>) {
        let dto = CreateAttachmentDto {
            project_id: self.project_id,
            task_id: self.task_id,
            milestone_id: self.milestone_id,
            change_request_id: self.change_request_id,
            original_name: file.original_name,
            mime_type: file.mime_type,
        };

        (dto, file.bytes)
    }
}

#[derive(Debug)]
pub struct AttachmentRawFileDto {
    pub mime_type: String,
    pub content_disposition: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum AttachmentViewer {
    User { user_id: Uuid },
    Client { user_id: Uuid, client_id: Uuid },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentDeleteResponseDto {
    pub message: String,
}

pub struct UploadedFile {
    pub original_name: String,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}
