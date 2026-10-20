use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::dtos::task::TaskStatus;

/// Estado de tarea del usuario. `category` es el significado para el sistema; `key`
/// identifica los estados por defecto (para mostrarlos traducidos mientras no se renombren).
#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct TaskStatusDto {
    pub id: Uuid,
    pub key: Option<String>,
    pub name: String,
    pub category: String,
    pub color: String,
    pub position: i32,
    pub task_count: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveTaskStatusDto {
    #[validate(length(min = 1, max = 50, message = "status name must be between 1 and 50 characters"))]
    pub name: String,
    pub category: TaskStatus,
    #[validate(length(min = 4, max = 7))]
    pub color: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReorderTaskStatusesDto {
    #[validate(length(min = 1, max = 100))]
    pub ids: Vec<Uuid>,
}
