use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::activities,
    utils::search::FilterField,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ActivitySearchField {
    Id,
    Name,
    CreatedAt,
}

impl ActivitySearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::CreatedAt => "created_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::Name => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<activities::Column> {
        match self {
            Self::Id => FilterField::Uuid(activities::Column::Id),
            Self::Name => FilterField::Text(activities::Column::Name),
            Self::CreatedAt => FilterField::DateTime(activities::Column::CreatedAt),
        }
    }

    pub const fn order_column(self) -> activities::Column {
        match self {
            Self::Id => activities::Column::Id,
            Self::Name => activities::Column::Name,
            Self::CreatedAt => activities::Column::CreatedAt,
        }
    }
}

pub fn activity_filter_schema() -> FilterSchema {
    ActivitySearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateActivityDto {
    #[validate(length(min = 1, max = 100, message = "activity name must be between 1 and 100 characters"))]
    pub name: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateActivityDto {
    #[validate(length(min = 1, max = 100, message = "activity name must be between 1 and 100 characters"))]
    pub name: String,
}

#[derive(Serialize, FromQueryResult, ToSchema)]
pub struct ActivitySummaryDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Serialize, ToSchema)]
pub struct ActivityDetailDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
