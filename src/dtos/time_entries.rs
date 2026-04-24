use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{FromQueryResult, prelude::Decimal};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::time_entries,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum TimeEntrySearchField {
    Id,
    TaskId,
    Activity,
    InvoiceId,
    Billable,
    StartTime,
    EndTime,
    DurationMinutes,
    WorkDate,
    CreatedAt,
    UpdatedAt,
}

impl TimeEntrySearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::TaskId => "task_id",
            Self::Activity => "activity",
            Self::InvoiceId => "invoice_id",
            Self::Billable => "billable",
            Self::StartTime => "start_time",
            Self::EndTime => "end_time",
            Self::DurationMinutes => "duration_minutes",
            Self::WorkDate => "work_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::TaskId => ColumnType::Text,
            Self::Activity => ColumnType::Text,
            Self::InvoiceId => ColumnType::Text,
            Self::Billable => ColumnType::Boolean,
            Self::StartTime => ColumnType::Date,
            Self::EndTime => ColumnType::Date,
            Self::DurationMinutes => ColumnType::Number,
            Self::WorkDate => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<time_entries::Column> {
        match self {
            Self::Id => FilterField::Uuid(time_entries::Column::Id),
            Self::TaskId => FilterField::Uuid(time_entries::Column::TaskId),
            Self::Activity => FilterField::Uuid(time_entries::Column::Activity),
            Self::InvoiceId => FilterField::Uuid(time_entries::Column::InvoiceId),
            Self::Billable => FilterField::Boolean(time_entries::Column::Billable),
            Self::StartTime => FilterField::DateTime(time_entries::Column::StartTime),
            Self::EndTime => FilterField::DateTime(time_entries::Column::EndTime),
            Self::DurationMinutes => FilterField::NumberI64(time_entries::Column::DurationMinutes),
            Self::WorkDate => FilterField::Date(time_entries::Column::WorkDate),
            Self::CreatedAt => FilterField::DateTime(time_entries::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(time_entries::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> time_entries::Column {
        match self {
            Self::Id => time_entries::Column::Id,
            Self::TaskId => time_entries::Column::TaskId,
            Self::Activity => time_entries::Column::Activity,
            Self::InvoiceId => time_entries::Column::InvoiceId,
            Self::Billable => time_entries::Column::Billable,
            Self::StartTime => time_entries::Column::StartTime,
            Self::EndTime => time_entries::Column::EndTime,
            Self::DurationMinutes => time_entries::Column::DurationMinutes,
            Self::WorkDate => time_entries::Column::WorkDate,
            Self::CreatedAt => time_entries::Column::CreatedAt,
            Self::UpdatedAt => time_entries::Column::UpdatedAt,
        }
    }
}

pub fn time_entries_filter_schema() -> FilterSchema {
    TimeEntrySearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

const fn default_billable() -> bool {
    true
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTimeEntryDto {
    pub task_id: Uuid,
    pub activity: Uuid,
    #[serde(default = "default_billable")]
    pub billable: bool,

    #[validate(range(min = 1, message = "duration_minutes must be at least 1"))]
    pub duration_minutes: i32,

    pub work_date: Option<NaiveDate>,

    #[validate(length(max = 1000, message = "notes must be at most 1000 characters"))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StartTimeEntryTimerDto {
    pub task_id: Uuid,
    pub activity: Uuid,
    #[serde(default = "default_billable")]
    pub billable: bool,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub work_date: Option<NaiveDate>,

    #[validate(length(max = 1000, message = "notes must be at most 1000 characters"))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTimeEntryDto {
    pub task_id: Uuid,
    pub activity: Uuid,
    #[serde(default = "default_billable")]
    pub billable: bool,

    #[validate(range(min = 1, message = "duration_minutes must be at least 1"))]
    pub duration_minutes: i32,

    pub work_date: NaiveDate,

    #[validate(length(max = 1000, message = "notes must be at most 1000 characters"))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StopTimeEntryTimerDto {
    pub end_time: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TimeEntrySummaryDto {
    pub id: Uuid,
    pub task_id: Uuid,
    pub activity: Uuid,
    pub invoice_id: Option<Uuid>,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub end_time: Option<DateTime<FixedOffset>>,
    pub duration_minutes: i32,
    pub work_date: NaiveDate,
    pub billable: bool,
    pub applied_rate: Option<Decimal>,
    pub notes: Option<String>,
    pub task_title: Option<String>,
    pub activity_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct TimeEntryDetailDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub activity: Uuid,
    pub invoice_id: Option<Uuid>,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub end_time: Option<DateTime<FixedOffset>>,
    pub duration_minutes: i32,
    pub work_date: NaiveDate,
    pub billable: bool,
    pub applied_rate: Option<Decimal>,
    pub notes: Option<String>,
    pub task_title: Option<String>,
    pub activity_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, FromQueryResult)]
pub struct TimeEntryDetailRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_id: Uuid,
    pub activity: Uuid,
    pub invoice_id: Option<Uuid>,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub end_time: Option<DateTime<FixedOffset>>,
    pub duration_minutes: i32,
    pub work_date: NaiveDate,
    pub billable: bool,
    pub applied_rate: Option<Decimal>,
    pub notes: Option<String>,
    pub task_title: Option<String>,
    pub activity_name: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
