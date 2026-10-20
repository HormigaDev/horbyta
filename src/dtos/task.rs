use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        hex_color::HexColor,
        query::{ColumnType, FilterSchema},
    },
    entities::tasks,
    string_enum,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum TaskSearchField {
    Id,
    RecordNumber,
    ProjectId,
    ParentId,
    MilestoneId,
    Title,
    Status,
    Priority,
    Complexity,
    BlockParent,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

impl TaskSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::RecordNumber => "record_number",
            Self::ProjectId => "project_id",
            Self::ParentId => "parent_id",
            Self::MilestoneId => "milestone_id",
            Self::Title => "title",
            Self::Status => "status",
            Self::Priority => "priority",
            Self::Complexity => "complexity",
            Self::BlockParent => "block_parent",
            Self::DueDate => "due_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::RecordNumber => ColumnType::Number,
            Self::ProjectId => ColumnType::Text,
            Self::ParentId => ColumnType::Text,
            Self::MilestoneId => ColumnType::Text,
            Self::Title => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::Priority => ColumnType::Number,
            Self::Complexity => ColumnType::Number,
            Self::BlockParent => ColumnType::Boolean,
            Self::DueDate => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<tasks::Column> {
        match self {
            Self::Id => FilterField::Uuid(tasks::Column::Id),
            Self::RecordNumber => FilterField::NumberI64(tasks::Column::RecordNumber),
            Self::ProjectId => FilterField::Uuid(tasks::Column::ProjectId),
            Self::ParentId => FilterField::Uuid(tasks::Column::ParentId),
            Self::MilestoneId => FilterField::Uuid(tasks::Column::MilestoneId),
            Self::Title => FilterField::Text(tasks::Column::Title),
            Self::Status => FilterField::Text(tasks::Column::Status),
            Self::Priority => FilterField::NumberI16(tasks::Column::Priority),
            Self::Complexity => FilterField::NumberI16(tasks::Column::Complexity),
            Self::BlockParent => FilterField::Boolean(tasks::Column::BlockParent),
            Self::DueDate => FilterField::DateTime(tasks::Column::DueDate),
            Self::CreatedAt => FilterField::DateTime(tasks::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(tasks::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> tasks::Column {
        match self {
            Self::Id => tasks::Column::Id,
            Self::RecordNumber => tasks::Column::RecordNumber,
            Self::ProjectId => tasks::Column::ProjectId,
            Self::ParentId => tasks::Column::ParentId,
            Self::MilestoneId => tasks::Column::MilestoneId,
            Self::Title => tasks::Column::Title,
            Self::Status => tasks::Column::Status,
            Self::Priority => tasks::Column::Priority,
            Self::Complexity => tasks::Column::Complexity,
            Self::BlockParent => tasks::Column::BlockParent,
            Self::DueDate => tasks::Column::DueDate,
            Self::CreatedAt => tasks::Column::CreatedAt,
            Self::UpdatedAt => tasks::Column::UpdatedAt,
        }
    }
}

pub fn task_filter_schema() -> FilterSchema {
    TaskSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateTaskDto {
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255, message = "task title must be between 1 and 255 characters"))]
    pub title: String,

    #[validate(length(max = 15000, message = "task description must be at most 15000 characters"))]
    pub description: Option<String>,

    /// Estado elegido por el usuario. Si falta, se usa `status` (categoría) y la base
    /// asigna el primer estado de esa categoría; sin ninguno de los dos, "todo".
    #[serde(default)]
    pub status_id: Option<Uuid>,
    #[serde(default)]
    pub status: Option<TaskStatus>,

    #[validate(range(min = 1, max = 4, message = "task priority must be between 1 and 4"))]
    pub priority: i16,

    #[validate(range(min = 1, max = 4, message = "task complexity must be between 1 and 4"))]
    pub complexity: i16,

    pub block_parent: bool,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub tag_ids: Vec<Uuid>,

    /// Origen en el alcance del proyecto, decidido al crear la tarea. Solo aplica a
    /// proyectos con acuerdo aceptado y a tareas raíz (las subtareas heredan el origen).
    #[serde(default)]
    pub scope_origin: Option<TaskScopeOriginDto>,
}

/// `change_request_id` nulo: incluida en el alcance aceptado; con valor: extra de esa solicitud.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TaskScopeOriginDto {
    pub change_request_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateTaskDto {
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255, message = "task title must be between 1 and 255 characters"))]
    pub title: String,

    #[validate(length(max = 15000, message = "task description must be at most 15000 characters"))]
    pub description: Option<String>,

    /// Estado elegido por el usuario. Si falta, se usa `status` (categoría) y la base
    /// asigna el primer estado de esa categoría; sin ninguno de los dos, "todo".
    #[serde(default)]
    pub status_id: Option<Uuid>,
    #[serde(default)]
    pub status: Option<TaskStatus>,

    #[validate(range(min = 1, max = 4, message = "task priority must be between 1 and 4"))]
    pub priority: i16,

    #[validate(range(min = 1, max = 4, message = "task complexity must be between 1 and 4"))]
    pub complexity: i16,

    pub block_parent: bool,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateTaskStatusDto {
    #[serde(default)]
    pub status_id: Option<Uuid>,
    #[serde(default)]
    pub status: Option<TaskStatus>,
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct TaskSummaryDto {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub title: String,
    pub status: TaskStatus,
    pub status_id: Uuid,
    pub status_name: Option<String>,
    pub status_color: Option<String>,
    pub status_key: Option<String>,
    pub priority: i16,
    pub complexity: i16,
    pub due_date: Option<DateTime<FixedOffset>>,
    pub project_name: Option<String>,
    pub milestone_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskDetailDto {
    pub id: Uuid,
    pub record_number: i64,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub status_id: Uuid,
    pub status_name: Option<String>,
    pub status_color: Option<String>,
    pub status_key: Option<String>,
    pub priority: i16,
    pub complexity: i16,
    pub block_parent: bool,
    pub due_date: Option<DateTime<FixedOffset>>,
    pub project_name: Option<String>,
    pub milestone_name: Option<String>,
    pub parent_title: Option<String>,
    pub parent_record_number: Option<i64>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tag_ids: Vec<Uuid>,
    pub tags: Vec<TaskTagDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskProjectDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskTagDto {
    pub id: Uuid,
    pub name: String,
    pub color: HexColor,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilestoneLookupDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub due_date: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskViewDto {
    pub items: Vec<TaskSummaryDto>,
    pub total: u64,
    pub projects: Vec<TaskProjectDto>,
    pub milestones: Vec<MilestoneLookupDto>,
    pub tags: Vec<TaskTagDto>,
}

string_enum!(
    pub enum TaskStatus {
        Todo,
        InProgress,
        Review,
        Done,
        Cancelled,
    }
);
