use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Deserializer};
use validator::ValidationError;

use crate::error::AppError;

pub fn is_numeric(s: &str) -> Result<(), ValidationError> {
    if s.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ValidationError::new("must_be_numeric"))
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("password_min_length"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ValidationError::new("password_requires_lowercase"));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ValidationError::new("password_requires_uppercase"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("password_requires_number"));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ValidationError::new("password_requires_symbol"));
    }

    Ok(())
}

pub fn is_non_negative_decimal(decimal: &Decimal) -> Result<(), ValidationError> {
    if decimal < &Decimal::ZERO {
        return Err(ValidationError::new("non_negative_decimal"));
    }

    Ok(())
}

pub fn is_positive_decimal(decimal: &Decimal) -> Result<(), ValidationError> {
    if decimal <= &Decimal::ZERO {
        return Err(ValidationError::new("strict_positive_decimal"));
    }

    Ok(())
}

pub fn deserialize_optional_naive_date_from_string_or_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;

    match raw.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(value) => {
            if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                return Ok(Some(date));
            }

            if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
                return Ok(Some(datetime.date_naive()));
            }

            if let Ok(datetime) = DateTime::<FixedOffset>::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f%:z") {
                return Ok(Some(datetime.date_naive()));
            }

            Err(serde::de::Error::custom(
                "date must be in YYYY-MM-DD or ISO 8601 datetime format",
            ))
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
