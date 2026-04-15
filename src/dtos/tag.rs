use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        hex_color::HexColor,
        query::{ColumnType, FilterSchema},
    },
    entities::tags,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum TagSearchField {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}

impl TagSearchField {
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

    pub const fn filter_field(self) -> FilterField<tags::Column> {
        match self {
            Self::Id => FilterField::Uuid(tags::Column::Id),
            Self::Name => FilterField::Text(tags::Column::Name),
            Self::CreatedAt => FilterField::DateTime(tags::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(tags::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> tags::Column {
        match self {
            Self::Id => tags::Column::Id,
            Self::Name => tags::Column::Name,
            Self::CreatedAt => tags::Column::CreatedAt,
            Self::UpdatedAt => tags::Column::UpdatedAt,
        }
    }
}

pub fn tag_filter_schema() -> FilterSchema {
    TagSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct CreateTagDto {
    #[validate(length(
        min = 1,
        max = 50,
        message = "El nombre de la etiqueta debe tener entre 1 y 50 caracteres."
    ))]
    pub name: String,
    pub color: HexColor,
}

#[derive(Validate, Serialize, Deserialize, Debug)]
pub struct UpdateTagDto {
    #[validate(length(
        min = 1,
        max = 50,
        message = "El nombre de la etiqueta debe tener entre 1 y 50 caracteres."
    ))]
    pub name: String,
    pub color: HexColor,
}

#[derive(Serialize, FromQueryResult)]
pub struct TagSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct TagDetailDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}
