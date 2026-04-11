use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::dtos::query::{ColumnType, FilterSchema};

pub fn task_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("project_id", ColumnType::Text)
        .add_column("parent_id", ColumnType::Text)
        .add_column("milestone_id", ColumnType::Text)
        .add_column("title", ColumnType::Text)
        .add_column("status", ColumnType::Text)
        .add_column("priority", ColumnType::Number)
        .add_column("complexity", ColumnType::Number)
        .add_column("block_parent", ColumnType::Boolean)
        .add_column("due_date", ColumnType::Date)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

fn validate_status(status: &str) -> Result<(), ValidationError> {
    match status {
        "todo" | "in_progress" | "review" | "done" => Ok(()),
        _ => Err(ValidationError::new("invalid_status")),
    }
}

fn default_status() -> String {
    "todo".to_string()
}

fn default_priority() -> i16 {
    1
}

fn default_complexity() -> i16 {
    1
}

fn default_block_parent() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTaskDto {
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255, message = "El título debe tener entre 1 y 255 caracteres"))]
    pub title: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    #[serde(default = "default_status")]
    #[validate(custom(function = "validate_status", message = "Estado inválido"))]
    pub status: String,

    #[serde(default = "default_priority")]
    #[validate(range(min = 1, max = 4, message = "La prioridad debe estar entre 1 y 4"))]
    pub priority: i16,

    #[serde(default = "default_complexity")]
    #[validate(range(min = 1, max = 4, message = "La complejidad debe estar entre 1 y 4"))]
    pub complexity: i16,

    #[serde(default = "default_block_parent")]
    pub block_parent: bool,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTaskDto {
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,

    #[validate(length(min = 1, max = 255, message = "El título debe tener entre 1 y 255 caracteres"))]
    pub title: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    #[validate(custom(function = "validate_status", message = "Estado inválido"))]
    pub status: String,

    #[validate(range(min = 1, max = 4, message = "La prioridad debe estar entre 1 y 4"))]
    pub priority: i16,

    #[validate(range(min = 1, max = 4, message = "La complejidad debe estar entre 1 y 4"))]
    pub complexity: i16,

    pub block_parent: bool,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTaskStatusDto {
    #[validate(custom(function = "validate_status", message = "Estado inválido"))]
    pub status: String,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TaskSummaryDto {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub title: String,
    pub status: String,
    pub priority: i16,
    pub complexity: i16,
    pub due_date: Option<DateTime<FixedOffset>>,
    pub project_name: Option<String>,
    pub milestone_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct TaskDetailDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i16,
    pub complexity: i16,
    pub block_parent: bool,
    pub due_date: Option<DateTime<FixedOffset>>,
    pub project_name: Option<String>,
    pub milestone_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tag_ids: Vec<Uuid>,
    pub tags: Vec<LookupOptionDto>,
}

#[derive(Debug, Serialize)]
pub struct LookupOptionDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MilestoneLookupDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub due_date: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize)]
pub struct TaskViewDto {
    pub items: Vec<TaskSummaryDto>,
    pub total: u64,
    pub projects: Vec<LookupOptionDto>,
    pub milestones: Vec<MilestoneLookupDto>,
    pub tags: Vec<LookupOptionDto>,
}
