use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    string_enum,
};

pub fn project_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("name", ColumnType::Text)
        .add_column("category_id", ColumnType::Text)
        .add_column("client_id", ColumnType::Text)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
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
    pub name: String,
    pub status: ProjectStatus,
    pub category_name: Option<String>,
    pub client_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Serialize, FromQueryResult)]
pub struct ProjectDetailDto {
    pub id: Uuid,
    pub name: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub client_id: Option<String>,
    pub client_name: Option<String>,
    pub description: Option<String>,
    pub status: ProjectStatus,
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
    pub name: String,
    pub status: ProjectStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

pub struct ClientProjectDetailDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
