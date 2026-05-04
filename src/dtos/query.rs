use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub enum FilterOperator {
    #[serde(rename = "=")]
    Eq,
    #[serde(rename = "!=")]
    Neq,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = ">=")]
    Gte,
    #[serde(rename = "<=")]
    Lte,
    #[serde(rename = "><")]
    Contains,
    #[serde(rename = "<>")]
    NotContains,
    #[serde(rename = "()")]
    Between,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct Filter {
    pub column: String,
    pub operator: FilterOperator,
    #[schema(value_type = Object)]
    pub value: Value,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryPayload {
    #[validate(range(min = 1))]
    #[serde(default = "default_page")]
    pub page: u64,

    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    pub limit: u64,

    pub order_by: Option<String>,

    #[serde(default = "default_order_asc")]
    pub order_asc: bool,

    #[validate(length(max = 5, message = "at most 5 filters are allowed"))]
    #[serde(default)]
    pub filters: Vec<Filter>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
}

fn default_page() -> u64 {
    1
}
fn default_limit() -> u64 {
    10
}
fn default_order_asc() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Text,
    Number,
    Boolean,
    Date,
}

pub struct FilterSchema {
    pub allowed_columns: HashMap<&'static str, ColumnType>,
}

impl FilterSchema {
    pub fn new() -> Self {
        Self {
            allowed_columns: HashMap::new(),
        }
    }

    pub fn add_column(mut self, name: &'static str, col_type: ColumnType) -> Self {
        self.allowed_columns.insert(name, col_type);
        self
    }

    /// Valida dinámicamente un QueryPayload contra este Schema.
    /// Devuelve AppError::BadRequest si encuentra una anomalía.
    pub fn validate_payload(&self, payload: &QueryPayload) -> Result<(), AppError> {
        crate::utils::search::validate_payload_against_schema(self, payload)
    }
}
