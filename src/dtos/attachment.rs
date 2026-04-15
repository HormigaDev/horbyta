use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::attachments,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum AttachmentSearchField {
    Id,
    ProjectId,
    TaskId,
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

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct CreateAttachmentDto {
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "El nombre original del archivo debe tener entre 1 y 255 caracteres"
    ))]
    pub original_name: String,

    #[validate(length(min = 1, max = 100, message = "El MIME type debe tener entre 1 y 100 caracteres"))]
    pub mime_type: Option<String>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct AttachmentSummaryDto {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentDetailDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub original_name: String,
    pub storage_key: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentStorageUsageDto {
    pub used_bytes: i64,
    pub limit_bytes: i64,
    pub remaining_bytes: i64,
}

pub struct UploadedFile {
    pub original_name: String,
    pub mime_type: Option<String>,
    pub bytes: Vec<u8>,
}
