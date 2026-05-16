use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::entities::user_ui_state::Model as UiStateModel;

const MAX_VALUE_BYTES: usize = 16 * 1024;

fn validate_scope(scope: &str) -> Result<(), validator::ValidationError> {
    if scope.is_empty() {
        return Err(validator::ValidationError::new("scope cannot be empty"));
    }

    let is_valid = scope
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !is_valid {
        return Err(validator::ValidationError::new(
            "scope must contain only lowercase letters, digits or underscores",
        ));
    }

    Ok(())
}

fn validate_key(key: &str) -> Result<(), validator::ValidationError> {
    if key.is_empty() {
        return Err(validator::ValidationError::new("key cannot be empty"));
    }

    let is_valid = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':');

    if !is_valid {
        return Err(validator::ValidationError::new(
            "key must contain only ascii letters, digits, '_', '-', '.' or ':'",
        ));
    }

    Ok(())
}

pub fn assert_value_within_limit(value: &serde_json::Value) -> Result<(), crate::error::AppError> {
    let serialized = serde_json::to_vec(value)
        .map_err(|_| crate::error::AppError::Internal("could not serialize ui state value".to_string()))?;

    if serialized.len() > MAX_VALUE_BYTES {
        return Err(crate::error::AppError::BadRequest(format!(
            "ui state value exceeds the {MAX_VALUE_BYTES} byte limit"
        )));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UiStateEntryDto {
    pub scope: String,
    pub key: String,
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<UiStateModel> for UiStateEntryDto {
    fn from(model: UiStateModel) -> Self {
        Self {
            scope: model.scope,
            key: model.key,
            value: model.value,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListScopeQuery {
    pub scope: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpsertUiStateDto {
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchUpsertEntryDto {
    pub key: String,
    #[schema(value_type = Object)]
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BatchUpsertUiStateDto {
    #[validate(length(min = 1, max = 50, message = "batch must contain between 1 and 50 entries"))]
    pub entries: Vec<BatchUpsertEntryDto>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UiStatePathParams {
    #[validate(length(min = 1, max = 40))]
    #[validate(custom(function = "validate_scope_param"))]
    pub scope: String,

    #[validate(length(min = 1, max = 120))]
    #[validate(custom(function = "validate_key_param"))]
    pub key: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UiStateScopeParam {
    #[validate(length(min = 1, max = 40))]
    #[validate(custom(function = "validate_scope_param"))]
    pub scope: String,
}

fn validate_scope_param(scope: &str) -> Result<(), validator::ValidationError> {
    validate_scope(scope)
}

fn validate_key_param(key: &str) -> Result<(), validator::ValidationError> {
    validate_key(key)
}
