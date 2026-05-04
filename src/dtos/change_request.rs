use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::change_requests as cr,
    string_enum,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ChangeRequestSearchField {
    Id,
    RecordNumber,
    ProjectId,
    ClientId,
    TaskId,
    Title,
    Origin,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl ChangeRequestSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::RecordNumber => "record_number",
            Self::ProjectId => "project_id",
            Self::ClientId => "client_id",
            Self::TaskId => "task_id",
            Self::Title => "title",
            Self::Origin => "origin",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::RecordNumber => ColumnType::Number,
            Self::ProjectId => ColumnType::Text,
            Self::ClientId => ColumnType::Text,
            Self::TaskId => ColumnType::Text,
            Self::Title => ColumnType::Text,
            Self::Origin => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<cr::Column> {
        match self {
            Self::Id => FilterField::Uuid(cr::Column::Id),
            Self::RecordNumber => FilterField::NumberI64(cr::Column::RecordNumber),
            Self::ProjectId => FilterField::Uuid(cr::Column::ProjectId),
            Self::ClientId => FilterField::Uuid(cr::Column::ClientId),
            Self::TaskId => FilterField::Uuid(cr::Column::TaskId),
            Self::Title => FilterField::Text(cr::Column::Title),
            Self::Origin => FilterField::Keyword(cr::Column::Origin),
            Self::Status => FilterField::Keyword(cr::Column::Status),
            Self::CreatedAt => FilterField::DateTime(cr::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(cr::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> cr::Column {
        match self {
            Self::Id => cr::Column::Id,
            Self::RecordNumber => cr::Column::RecordNumber,
            Self::ProjectId => cr::Column::ProjectId,
            Self::ClientId => cr::Column::ClientId,
            Self::TaskId => cr::Column::TaskId,
            Self::Title => cr::Column::Title,
            Self::Origin => cr::Column::Origin,
            Self::Status => cr::Column::Status,
            Self::CreatedAt => cr::Column::CreatedAt,
            Self::UpdatedAt => cr::Column::UpdatedAt,
        }
    }
}

pub fn change_request_filter_schema() -> FilterSchema {
    ChangeRequestSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ClientChangeRequestSearchField {
    Id,
    RecordNumber,
    ProjectId,
    TaskId,
    Title,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl ClientChangeRequestSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::RecordNumber => "record_number",
            Self::ProjectId => "project_id",
            Self::TaskId => "task_id",
            Self::Title => "title",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::RecordNumber => ColumnType::Number,
            Self::ProjectId => ColumnType::Text,
            Self::TaskId => ColumnType::Text,
            Self::Title => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<cr::Column> {
        match self {
            Self::Id => FilterField::Uuid(cr::Column::Id),
            Self::RecordNumber => FilterField::NumberI64(cr::Column::RecordNumber),
            Self::ProjectId => FilterField::Uuid(cr::Column::ProjectId),
            Self::TaskId => FilterField::Uuid(cr::Column::TaskId),
            Self::Title => FilterField::Text(cr::Column::Title),
            Self::Status => FilterField::Keyword(cr::Column::Status),
            Self::CreatedAt => FilterField::DateTime(cr::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(cr::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> cr::Column {
        match self {
            Self::Id => cr::Column::Id,
            Self::RecordNumber => cr::Column::RecordNumber,
            Self::ProjectId => cr::Column::ProjectId,
            Self::TaskId => cr::Column::TaskId,
            Self::Title => cr::Column::Title,
            Self::Status => cr::Column::Status,
            Self::CreatedAt => cr::Column::CreatedAt,
            Self::UpdatedAt => cr::Column::UpdatedAt,
        }
    }
}

pub fn client_change_request_filter_schema() -> FilterSchema {
    ClientChangeRequestSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

fn validate_non_negative_decimal(value: &Decimal) -> Result<(), ValidationError> {
    if *value < Decimal::ZERO {
        return Err(ValidationError::new("invalid_decimal_range"));
    }

    Ok(())
}

fn validate_signature_ip(ip_raw: &str) -> Result<(), ValidationError> {
    if ip_raw.is_empty() {
        return Err(ValidationError::new("invalid_signature_ip"));
    }

    if ip_raw.len() > 45 {
        return Err(ValidationError::new("invalid_signature_ip"));
    }

    if ip_raw.parse::<IpAddr>().is_err() {
        return Err(ValidationError::new("invalid_signature_ip"));
    }

    Ok(())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateChangeRequestDto {
    pub project_id: Uuid,
    pub client_id: Uuid,
    pub task_id: Option<Uuid>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "change request title must be between 1 and 255 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 20000,
        message = "client request description must be between 1 and 20000 characters"
    ))]
    pub client_request_description: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateChangeRequestDto {
    pub project_id: Uuid,
    pub client_id: Uuid,
    pub task_id: Option<Uuid>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "change request title must be between 1 and 255 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 20000,
        message = "client request description must be between 1 and 20000 characters"
    ))]
    pub client_request_description: String,

    #[validate(length(max = 20000, message = "technical assessment must be at most 20000 characters"))]
    pub technical_assessment: Option<String>,

    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub impact_hours: Decimal,

    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub impact_cost: Decimal,

    #[validate(range(min = 0, message = "schedule impact days must be at least 0"))]
    pub schedule_impact_days: i32,

    #[validate(range(min = 1, message = "version must be at least 1"))]
    pub version: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssessChangeRequestDto {
    #[validate(length(
        min = 1,
        max = 20000,
        message = "technical assessment must be between 1 and 20000 characters"
    ))]
    pub technical_assessment: String,

    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub impact_hours: Decimal,

    #[validate(custom(function = "validate_non_negative_decimal"))]
    pub impact_cost: Decimal,

    #[validate(range(min = 0, message = "schedule impact days must be at least 0"))]
    pub schedule_impact_days: i32,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AssignChangeRequestTaskDto {
    pub task_id: Option<Uuid>,

    #[validate(range(min = 1, message = "version must be at least 1"))]
    pub version: i32,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientCreateChangeRequestDto {
    pub project_id: Uuid,

    #[validate(length(
        min = 1,
        max = 255,
        message = "change request title must be between 1 and 255 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 20000,
        message = "client request description must be between 1 and 20000 characters"
    ))]
    pub client_request_description: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientUpdateChangeRequestDto {
    #[validate(length(
        min = 1,
        max = 255,
        message = "change request title must be between 1 and 255 characters"
    ))]
    pub title: String,

    #[validate(length(
        min = 1,
        max = 20000,
        message = "client request description must be between 1 and 20000 characters"
    ))]
    pub client_request_description: String,

    #[validate(range(min = 1, message = "version must be at least 1"))]
    pub version: i32,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientCancelChangeRequestDto {
    #[validate(range(min = 1, message = "version must be at least 1"))]
    pub version: i32,
}

#[derive(Deserialize, Validate, Default, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientApproveChangeRequestDto {
    #[validate(custom(function = "validate_signature_ip"))]
    pub client_signature_ip: Option<String>,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientDeclineChangeRequestDto {
    pub decline_reason: ChangeRequestDeclineReason,
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct ChangeRequestSummaryDto {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub origin: ChangeRequestOrigin,
    pub status: ChangeRequestStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChangeRequestDetailDto {
    pub id: Uuid,
    pub record_number: i64,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub client_request_description: String,
    pub technical_assessment: Option<String>,
    pub impact_hours: Decimal,
    pub impact_cost: Decimal,
    pub schedule_impact_days: i32,
    pub origin: ChangeRequestOrigin,
    pub status: ChangeRequestStatus,
    pub decline_reason: Option<ChangeRequestDeclineReason>,
    pub client_approved_at: Option<DateTime<FixedOffset>>,
    pub client_signature_ip: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub version: i32,
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct ClientChangeRequestSummaryDto {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub status: ChangeRequestStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientChangeRequestDetailDto {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub client_request_description: String,
    pub technical_assessment: Option<String>,
    pub impact_hours: Decimal,
    pub impact_cost: Decimal,
    pub schedule_impact_days: i32,
    pub origin: ChangeRequestOrigin,
    pub status: ChangeRequestStatus,
    pub decline_reason: Option<ChangeRequestDeclineReason>,
    pub client_approved_at: Option<DateTime<FixedOffset>>,
    pub client_signature_ip: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub version: i32,
}

#[derive(Debug, FromQueryResult)]
pub struct ChangeRequestDetailRow {
    pub id: Uuid,
    pub record_number: i64,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub client_id: Uuid,
    pub client_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub client_request_description: String,
    pub technical_assessment: Option<String>,
    pub impact_hours: Decimal,
    pub impact_cost: Decimal,
    pub schedule_impact_days: i32,
    pub origin: ChangeRequestOrigin,
    pub status: ChangeRequestStatus,
    pub decline_reason: Option<String>,
    pub client_approved_at: Option<DateTime<FixedOffset>>,
    pub client_signature_ip: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub version: i32,
}

#[derive(Debug, FromQueryResult)]
pub struct ClientChangeRequestDetailRow {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub title: String,
    pub client_request_description: String,
    pub technical_assessment: Option<String>,
    pub impact_hours: Decimal,
    pub impact_cost: Decimal,
    pub schedule_impact_days: i32,
    pub origin: ChangeRequestOrigin,
    pub status: ChangeRequestStatus,
    pub decline_reason: Option<String>,
    pub client_approved_at: Option<DateTime<FixedOffset>>,
    pub client_signature_ip: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub version: i32,
}

string_enum!(
    pub enum ChangeRequestStatus {
        Draft,
        Submitted,
        Assessed,
        Approved,
        Declined,
        Implemented,
        Cancelled,
    }
);

string_enum!(
    pub enum ChangeRequestOrigin {
        FreelancerDashboard,
        ClientPortal,
    }
);

string_enum!(
    pub enum ChangeRequestDeclineReason {
        BudgetExceeded,
        RoiNotJustified,
        ScheduleTooLong,
        JeopardizesMainDeadline,
        PostponedToNextPhase,
        RethinkingRequirements,
        NotNecessaryAnymore,
        WillSolveInternally,
        ThoughtItWasIncluded,
    }
);
