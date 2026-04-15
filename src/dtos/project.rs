use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        milestone::ProjectMilestoneSummaryDto,
        query::{ColumnType, FilterSchema},
    },
    entities::projects,
    string_enum,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ProjectSearchField {
    Id,
    ProjectNumber,
    Name,
    CategoryId,
    ClientId,
    CreatedAt,
    UpdatedAt,
}

impl ProjectSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::ProjectNumber => "project_number",
            Self::Name => "name",
            Self::CategoryId => "category_id",
            Self::ClientId => "client_id",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::ProjectNumber => ColumnType::Number,
            Self::Name => ColumnType::Text,
            Self::CategoryId => ColumnType::Text,
            Self::ClientId => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<projects::Column> {
        match self {
            Self::Id => FilterField::Uuid(projects::Column::Id),
            Self::ProjectNumber => FilterField::NumberI64(projects::Column::ProjectNumber),
            Self::Name => FilterField::Text(projects::Column::Name),
            Self::CategoryId => FilterField::Uuid(projects::Column::CategoryId),
            Self::ClientId => FilterField::Uuid(projects::Column::ClientId),
            Self::CreatedAt => FilterField::DateTime(projects::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(projects::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> projects::Column {
        match self {
            Self::Id => projects::Column::Id,
            Self::ProjectNumber => projects::Column::ProjectNumber,
            Self::Name => projects::Column::Name,
            Self::CategoryId => projects::Column::CategoryId,
            Self::ClientId => projects::Column::ClientId,
            Self::CreatedAt => projects::Column::CreatedAt,
            Self::UpdatedAt => projects::Column::UpdatedAt,
        }
    }
}

pub fn project_filter_schema() -> FilterSchema {
    ProjectSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ClientProjectSearchField {
    Id,
    ProjectNumber,
    Name,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl ClientProjectSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::ProjectNumber => "project_number",
            Self::Name => "name",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::ProjectNumber => ColumnType::Number,
            Self::Name => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<projects::Column> {
        match self {
            Self::Id => FilterField::Uuid(projects::Column::Id),
            Self::ProjectNumber => FilterField::NumberI64(projects::Column::ProjectNumber),
            Self::Name => FilterField::Text(projects::Column::Name),
            Self::Status => FilterField::Keyword(projects::Column::Status),
            Self::CreatedAt => FilterField::DateTime(projects::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(projects::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> projects::Column {
        match self {
            Self::Id => projects::Column::Id,
            Self::ProjectNumber => projects::Column::ProjectNumber,
            Self::Name => projects::Column::Name,
            Self::Status => projects::Column::Status,
            Self::CreatedAt => projects::Column::CreatedAt,
            Self::UpdatedAt => projects::Column::UpdatedAt,
        }
    }
}

pub fn client_project_filter_schema() -> FilterSchema {
    ClientProjectSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Deserialize, Validate, Clone)]
pub struct CreateMilestoneSubDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "El nombre del hito debe tener entre 1 y 100 caracteres."
    ))]
    pub name: String,
    pub due_date: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 1, max = 255, message = "El nombre debe tener entre 1 y 255 caracteres"))]
    pub name: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    pub status: Option<ProjectStatus>,

    pub category_id: Option<Uuid>,
    pub client_id: Option<Uuid>,

    #[validate(nested)]
    pub milestones: Option<Vec<CreateMilestoneSubDto>>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateProjectDto {
    #[validate(length(min = 1, max = 255, message = "El nombre debe tener entre 1 y 255 caracteres"))]
    pub name: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    pub status: ProjectStatus,

    pub category_id: Option<Uuid>,
    pub client_id: Option<Uuid>,

    #[validate(nested)]
    pub new_milestones: Option<Vec<CreateMilestoneSubDto>>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateProjectCategoryDto {
    pub category_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateProjectStatusDto {
    pub status: ProjectStatus,
}

#[derive(Serialize, FromQueryResult)]
pub struct ProjectSummaryDto {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub status: ProjectStatus,
    pub category_name: Option<String>,
    pub client_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Serialize)]
pub struct ProjectDetailDto {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub milestones: Vec<ProjectMilestoneSummaryDto>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

string_enum!(
    pub enum ProjectStatus {
        Open,
        Closed,
        Cancelled,
    }
);

#[derive(Serialize, FromQueryResult)]
pub struct ClientProjectSummaryDto {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub status: ProjectStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Serialize)]
pub struct ClientProjectDetailDto {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub milestones: Vec<ProjectMilestoneSummaryDto>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(FromQueryResult)]
pub struct ProjectDetailRow {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(FromQueryResult)]
pub struct ClientProjectDetailRow {
    pub id: Uuid,
    pub project_number: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
