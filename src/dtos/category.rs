use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::categories,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum CategorySearchField {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

impl CategorySearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::Name => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<categories::Column> {
        match self {
            Self::Id => FilterField::Uuid(categories::Column::Id),
            Self::Name => FilterField::Text(categories::Column::Name),
            Self::CreatedAt => FilterField::DateTime(categories::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(categories::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> categories::Column {
        match self {
            Self::Id => categories::Column::Id,
            Self::Name => categories::Column::Name,
            Self::CreatedAt => categories::Column::CreatedAt,
            Self::UpdatedAt => categories::Column::UpdatedAt,
        }
    }
}

pub fn category_filter_schema() -> FilterSchema {
    CategorySearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
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
