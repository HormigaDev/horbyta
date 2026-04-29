use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{
    FromQueryResult,
    prelude::{DateTimeWithTimeZone, Decimal},
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use super::utils::{deserialize_optional_naive_date_from_string_or_datetime, is_positive_decimal};
use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::payments,
    string_enum,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum PaymentSearchField {
    Id,
    RecordNumber,
    ProjectId,
    InvoiceId,
    ParentPaymentId,
    CurrencyType,
    TransactionType,
    PaymentDate,
    CreatedAt,
    UpdatedAt,
}

impl PaymentSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::RecordNumber => "record_number",
            Self::ProjectId => "project_id",
            Self::InvoiceId => "invoice_id",
            Self::ParentPaymentId => "parent_payment_id",
            Self::CurrencyType => "currency_type",
            Self::TransactionType => "transaction_type",
            Self::PaymentDate => "payment_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::RecordNumber => ColumnType::Number,
            Self::ProjectId => ColumnType::Text,
            Self::InvoiceId => ColumnType::Text,
            Self::ParentPaymentId => ColumnType::Text,
            Self::CurrencyType => ColumnType::Text,
            Self::TransactionType => ColumnType::Text,
            Self::PaymentDate => ColumnType::Date,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<payments::Column> {
        match self {
            Self::Id => FilterField::Uuid(payments::Column::Id),
            Self::RecordNumber => FilterField::NumberI64(payments::Column::RecordNumber),
            Self::ProjectId => FilterField::Uuid(payments::Column::ProjectId),
            Self::InvoiceId => FilterField::Uuid(payments::Column::InvoiceId),
            Self::ParentPaymentId => FilterField::Uuid(payments::Column::ParentPaymentId),
            Self::CurrencyType => FilterField::Text(payments::Column::CurrencyType),
            Self::TransactionType => FilterField::Keyword(payments::Column::TransactionType),
            Self::PaymentDate => FilterField::Date(payments::Column::PaymentDate),
            Self::CreatedAt => FilterField::DateTime(payments::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(payments::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> payments::Column {
        match self {
            Self::Id => payments::Column::Id,
            Self::RecordNumber => payments::Column::RecordNumber,
            Self::ProjectId => payments::Column::ProjectId,
            Self::InvoiceId => payments::Column::InvoiceId,
            Self::ParentPaymentId => payments::Column::ParentPaymentId,
            Self::CurrencyType => payments::Column::CurrencyType,
            Self::TransactionType => payments::Column::TransactionType,
            Self::PaymentDate => payments::Column::PaymentDate,
            Self::CreatedAt => payments::Column::CreatedAt,
            Self::UpdatedAt => payments::Column::UpdatedAt,
        }
    }
}

pub fn payment_filter_schema() -> FilterSchema {
    PaymentSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Deserialize, Validate)]
pub struct CreatePaymentDto {
    pub project_id: Uuid,
    pub invoice_id: Option<Uuid>,

    #[validate(custom(function = "is_positive_decimal"))]
    pub amount: Decimal,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_naive_date_from_string_or_datetime"
    )]
    pub payment_date: Option<NaiveDate>,

    #[validate(length(max = 50, message = "payment_method must be at most 50 characters"))]
    pub payment_method: Option<String>,

    #[validate(length(max = 255, message = "reference_code must be at most 255 characters"))]
    pub reference_code: Option<String>,

    #[validate(length(max = 5000, message = "notes must be at most 5000 characters"))]
    pub notes: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct CreateRefundDto {
    pub parent_payment_id: Uuid,

    #[validate(custom(function = "is_positive_decimal"))]
    pub amount: Decimal,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_naive_date_from_string_or_datetime"
    )]
    pub refund_date: Option<NaiveDate>,

    #[validate(length(max = 5000, message = "notes must be at most 5000 characters"))]
    pub notes: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdatePaymentDto {
    #[validate(length(max = 50, message = "payment_method must be at most 50 characters"))]
    pub payment_method: Option<String>,

    #[validate(length(max = 255, message = "reference_code must be at most 255 characters"))]
    pub reference_code: Option<String>,

    #[validate(length(max = 5000, message = "notes must be at most 5000 characters"))]
    pub notes: Option<String>,
}

#[derive(Serialize, FromQueryResult)]
pub struct PaymentSummaryDto {
    pub id: Uuid,
    pub record_number: i64,

    pub project_name: Option<String>,
    pub client_name: Option<String>,
    pub invoice_record_number: Option<i64>,

    pub amount: Decimal,

    pub currency_type: String,
    pub transaction_type: PaymentTransactionType,

    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub provider: String,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct PaymentDetailDto {
    pub id: Uuid,
    pub record_number: i64,

    pub project_id: Uuid,
    pub project_name: String,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,

    pub invoice: Option<PaymentInvoiceReferenceDto>,

    pub amount: Decimal,
    pub currency_type: String,
    pub transaction_type: PaymentTransactionType,
    pub status: String,

    pub related_refunds: Vec<RelatedRefundDto>,
    pub parent_payment: Option<ParentPaymentReferenceDto>,

    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub reference_code: Option<String>,
    pub provider: String,
    pub provider_external_id: Option<String>,
    pub notes: Option<String>,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct PaymentInvoiceReferenceDto {
    pub id: Uuid,
    pub record_number: i64,
    pub status: String,
}

#[derive(Serialize)]
pub struct RelatedRefundDto {
    pub id: Uuid,
    pub record_number: i64,
    pub amount: Decimal,
    pub payment_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct ParentPaymentReferenceDto {
    pub id: Uuid,
    pub record_number: i64,
    pub payment_date: NaiveDate,
}

string_enum!(
    pub enum PaymentTransactionType {
        Charge,
        Refund,
    }
);

#[derive(Debug, FromQueryResult)]
pub struct PaymentDetailRow {
    pub id: Uuid,
    pub record_number: i64,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub parent_payment_id: Option<Uuid>,
    pub amount: Decimal,
    pub currency_type: String,
    pub transaction_type: String,
    pub payment_date: NaiveDate,
    pub payment_method: Option<String>,
    pub reference_code: Option<String>,
    pub provider: String,
    pub provider_external_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
