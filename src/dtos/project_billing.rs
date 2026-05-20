use sea_orm::prelude::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use super::utils::is_non_negative_decimal;
use crate::string_enum;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateProjectBillingDto {
    #[validate(length(min = 3, max = 3, message = "the currency type must be 3 characters long"))]
    pub currency_type: String,

    pub billing_type: ProjectBillingType,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub hourly_rate: Option<Decimal>,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub fixed_price: Option<Decimal>,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub retainer_amount: Option<Decimal>,

    #[validate(range(min = 1, max = 28, message = "the payment day must be between 1 and 28"))]
    pub payment_day: Option<i16>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateProjectBillingDto {
    #[validate(length(min = 3, max = 3, message = "the currency type must be 3 characters long"))]
    pub currency_type: String,

    pub billing_type: ProjectBillingType,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub hourly_rate: Option<Decimal>,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub fixed_price: Option<Decimal>,

    #[validate(custom(function = "is_non_negative_decimal"))]
    pub retainer_amount: Option<Decimal>,

    #[validate(range(min = 1, max = 28, message = "the payment day must be between 1 and 28"))]
    pub payment_day: Option<i16>,
}

pub(crate) struct NormalizedProjectBilling {
    pub(crate) currency_type: String,
    pub(crate) billing_type: String,
    pub(crate) hourly_rate: Option<Decimal>,
    pub(crate) fixed_price: Option<Decimal>,
    pub(crate) retainer_amount: Option<Decimal>,
    pub(crate) payment_day: Option<i16>,
}

string_enum!(
    pub enum ProjectBillingType {
        Fixed,
        Hourly,
        Retainer,
    }
);
