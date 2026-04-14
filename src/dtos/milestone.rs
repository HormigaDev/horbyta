use chrono::DateTime;
use chrono::FixedOffset;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::milestones,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum MilestoneSearchField {
    Id,
    ProjectId,
    Name,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

impl MilestoneSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::ProjectId => "project_id",
            Self::Name => "name",
            Self::DueDate => "due_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::ProjectId => ColumnType::Text,
            Self::Name => ColumnType::Text,
            Self::DueDate => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<milestones::Column> {
        match self {
            Self::Id => FilterField::Uuid(milestones::Column::Id),
            Self::ProjectId => FilterField::Uuid(milestones::Column::ProjectId),
            Self::Name => FilterField::Text(milestones::Column::Name),
            Self::DueDate => FilterField::DateTime(milestones::Column::DueDate),
            Self::CreatedAt => FilterField::DateTime(milestones::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(milestones::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> milestones::Column {
        match self {
            Self::Id => milestones::Column::Id,
            Self::ProjectId => milestones::Column::ProjectId,
            Self::Name => milestones::Column::Name,
            Self::DueDate => milestones::Column::DueDate,
            Self::CreatedAt => milestones::Column::CreatedAt,
            Self::UpdatedAt => milestones::Column::UpdatedAt,
        }
    }
}

pub fn milestone_filter_schema() -> FilterSchema {
    MilestoneSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ClientMilestoneSearchField {
    Id,
    Name,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

impl ClientMilestoneSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::DueDate => "due_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::Name => ColumnType::Text,
            Self::DueDate => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<milestones::Column> {
        match self {
            Self::Id => FilterField::Uuid(milestones::Column::Id),
            Self::Name => FilterField::Text(milestones::Column::Name),
            Self::DueDate => FilterField::DateTime(milestones::Column::DueDate),
            Self::CreatedAt => FilterField::DateTime(milestones::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(milestones::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> milestones::Column {
        match self {
            Self::Id => milestones::Column::Id,
            Self::Name => milestones::Column::Name,
            Self::DueDate => milestones::Column::DueDate,
            Self::CreatedAt => milestones::Column::CreatedAt,
            Self::UpdatedAt => milestones::Column::UpdatedAt,
        }
    }
}

pub fn client_milestone_filter_schema() -> FilterSchema {
    ClientMilestoneSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
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

#[derive(Debug, Clone, Serialize, FromQueryResult)]
pub struct ProjectMilestoneSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub due_date: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
