use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dtos::query::{ColumnType, FilterSchema};

pub fn project_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("name", ColumnType::Text)
        .add_column("description", ColumnType::Text)
        .add_column("category_id", ColumnType::Text)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 1, max = 255, message = "El nombre debe tener entre 1 y 255 caracteres"))]
    pub name: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    pub category_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateProjectDto {
    #[validate(length(min = 1, max = 255, message = "El nombre debe tener entre 1 y 255 caracteres"))]
    pub name: String,

    #[validate(length(max = 15000, message = "La descripción no puede exceder los 15000 caracteres"))]
    pub description: Option<String>,

    pub category_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateProjectCategoryDto {
    pub category_id: Option<Uuid>,
}
