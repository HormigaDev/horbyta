use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{FromQueryResult, prelude::Decimal};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use super::utils::is_non_negative_decimal;
use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::invoices,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum InvoiceSearchField {
    Id,
    RecordNumber,
    ProjectId,
    CurrencyType,
    Status,
    DueDate,
    IssuedAt,
    CreatedAt,
    UpdatedAt,
}

impl InvoiceSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::RecordNumber => "record_number",
            Self::ProjectId => "project_id",
            Self::CurrencyType => "currency_type",
            Self::Status => "status",
            Self::DueDate => "due_date",
            Self::IssuedAt => "issued_at",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::RecordNumber => ColumnType::Number,
            Self::ProjectId => ColumnType::Text,
            Self::CurrencyType => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::DueDate => ColumnType::Date,
            Self::IssuedAt => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<invoices::Column> {
        match self {
            Self::Id => FilterField::Uuid(invoices::Column::Id),
            Self::RecordNumber => FilterField::NumberI64(invoices::Column::RecordNumber),
            Self::ProjectId => FilterField::Uuid(invoices::Column::ProjectId),
            Self::CurrencyType => FilterField::Text(invoices::Column::CurrencyType),
            Self::Status => FilterField::Text(invoices::Column::Status),
            Self::DueDate => FilterField::Date(invoices::Column::DueDate),
            Self::IssuedAt => FilterField::DateTime(invoices::Column::IssuedAt),
            Self::CreatedAt => FilterField::DateTime(invoices::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(invoices::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> invoices::Column {
        match self {
            Self::Id => invoices::Column::Id,
            Self::RecordNumber => invoices::Column::RecordNumber,
            Self::ProjectId => invoices::Column::ProjectId,
            Self::CurrencyType => invoices::Column::CurrencyType,
            Self::Status => invoices::Column::Status,
            Self::DueDate => invoices::Column::DueDate,
            Self::IssuedAt => invoices::Column::IssuedAt,
            Self::CreatedAt => invoices::Column::CreatedAt,
            Self::UpdatedAt => invoices::Column::UpdatedAt,
        }
    }
}

pub fn invoice_filter_schema() -> FilterSchema {
    InvoiceSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceDto {
    pub project_id: Uuid,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub amount: Decimal,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[validate(length(max = 5000, message = "freelancer_notes must be at most 5000 characters"))]
    pub freelancer_notes: Option<String>,

    #[validate(length(min = 1, message = "at least one time entry must be selected"))]
    pub time_entry_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInvoiceDto {
    #[validate(custom(function = "is_non_negative_decimal"))]
    pub amount: Decimal,

    pub due_date: Option<DateTime<FixedOffset>>,

    #[validate(length(max = 5000, message = "freelancer_notes must be at most 5000 characters"))]
    pub freelancer_notes: Option<String>,

    #[validate(length(min = 1, message = "at least one time entry must be selected"))]
    pub time_entry_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInvoiceStatusDto {
    pub status: String,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct InvoiceSummaryDto {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub client_name: Option<String>,
    pub amount: Decimal,
    pub currency_type: String,
    pub status: String,
    pub due_date: Option<NaiveDate>,
    pub issued_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceDetailDto {
    pub id: Uuid,
    pub record_number: i64,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub project_status: String,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub client_email: Option<String>,
    pub amount: Decimal,
    pub currency_type: String,
    pub status: String,
    pub due_date: Option<NaiveDate>,
    pub issued_at: Option<DateTime<FixedOffset>>,
    pub notes: Option<String>,
    pub freelancer_notes: Option<String>,
    pub task_groups: Vec<InvoiceTaskGroupDto>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceTaskGroupDto {
    pub task_id: Uuid,
    pub task_record_number: i64,
    pub task_title: String,
    pub total_duration_minutes: i64,
    pub time_entries: Vec<InvoiceTimeEntryDto>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct InvoiceTimeEntryDto {
    pub id: Uuid,
    pub task_id: Uuid,
    pub duration_minutes: i32,
    pub work_date: NaiveDate,
    pub billable: bool,
    pub applied_rate: Option<Decimal>,
    pub notes: Option<String>,
    pub activity_name: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub struct InvoiceDetailRow {
    pub id: Uuid,
    pub record_number: i64,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub project_status: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub client_email: Option<String>,
    pub amount: Decimal,
    pub currency_type: String,
    pub status: String,
    pub due_date: Option<NaiveDate>,
    pub issued_at: Option<DateTime<FixedOffset>>,
    pub notes: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct BillableTasksResponse {
    pub tasks: Vec<BillableTaskDto>,
    pub total_tasks: u64,
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Serialize)]
pub struct BillableTaskDto {
    pub task_id: Uuid,
    pub task_record_number: i64,
    pub task_title: String,
    pub total_duration_minutes: i64,
    pub time_entries: Vec<BillableTimeEntryDto>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct BillableTimeEntryDto {
    pub id: Uuid,
    pub task_id: Uuid,
    pub duration_minutes: i32,
    pub work_date: NaiveDate,
    pub billable: bool,
    pub applied_rate: Option<Decimal>,
    pub notes: Option<String>,
    pub activity_name: Option<String>,
}

#[derive(Debug, FromQueryResult)]
pub struct BillableTaskRow {
    pub task_id: Uuid,
    pub task_record_number: i64,
    pub task_title: String,
}

#[derive(Debug, FromQueryResult)]
pub struct BillableTimeEntryWithTask {
    pub id: Uuid,
    pub task_id: Uuid,
    pub duration_minutes: i32,
    pub applied_rate: Option<Decimal>,
    pub task_record_number: i64,
    pub task_title: String,
}
