use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dtos::query::{ColumnType, FilterSchema};

pub fn category_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("name", ColumnType::Text)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct CategoryDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "El nombre de la categoría debe tener entre 1 y 100 caracteres."
    ))]
    pub name: String,
}
