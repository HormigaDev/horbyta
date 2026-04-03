use chrono::DateTime;
use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::dtos::query::{ColumnType, FilterSchema};

pub fn milestone_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("project_id", ColumnType::Text)
        .add_column("name", ColumnType::Text)
        .add_column("due_date", ColumnType::Date)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct MilestoneDto {
    pub project_id: Uuid,
    #[validate(length(
        min = 1,
        max = 100,
        message = "El nombre del hito debe tener entre 1 y 100 caracteres."
    ))]
    pub name: String,
    pub due_date: Option<DateTime<FixedOffset>>,
}
