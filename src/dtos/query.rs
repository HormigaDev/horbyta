use crate::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Filter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: Value,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
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

    #[validate(length(max = 5, message = "Límite de filtros excedido."))]
    #[serde(default)]
    pub filters: Vec<Filter>,
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
        // Validar columna de ordenamiento si se envía
        if let Some(order_col) = &payload.order_by {
            if !self.allowed_columns.contains_key(order_col.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Columna de ordenamiento no permitida: {}",
                    order_col
                )));
            }
        }

        // Iterar hasta un máximo de 5 (el macro de validator ya atajó si son > 5)
        for filter in &payload.filters {
            let col_type = self
                .allowed_columns
                .get(filter.column.as_str())
                .ok_or_else(|| AppError::BadRequest(format!("Campo de filtro no permitido: {}", filter.column)))?;

            match filter.operator {
                FilterOperator::Contains | FilterOperator::NotContains => {
                    if !matches!(col_type, ColumnType::Text) {
                        return Err(AppError::BadRequest(format!(
                            "El operador '{:?}' solo es válido para campos de texto ({})",
                            filter.operator, filter.column
                        )));
                    }
                    if !filter.value.is_string() {
                        return Err(AppError::BadRequest(format!(
                            "El valor para '><' o '<>' en {} debe ser un string",
                            filter.column
                        )));
                    }
                }
                FilterOperator::Between => {
                    if !matches!(col_type, ColumnType::Date | ColumnType::Number) {
                        return Err(AppError::BadRequest(format!(
                            "El operador '()' solo es válido para fechas o números ({})",
                            filter.column
                        )));
                    }

                    if let Some(arr) = filter.value.as_array() {
                        if arr.len() != 2 {
                            return Err(AppError::BadRequest(format!(
                                "El operador '()' en {} requiere exactamente 2 valores en un array",
                                filter.column
                            )));
                        }
                        // En un setup más estricto validaríamos que los elementos calcen con el formato destino.
                    } else {
                        return Err(AppError::BadRequest(format!(
                            "El operador '()' en {} requiere un array de valores",
                            filter.column
                        )));
                    }
                }
                // Para booleanos solo Eq o Neq tiene sentido
                FilterOperator::Eq | FilterOperator::Neq => {
                    if matches!(col_type, ColumnType::Boolean) && !filter.value.is_boolean() {
                        return Err(AppError::BadRequest(format!("El campo {} es booleano", filter.column)));
                    }
                }
                // Validar que textos no usen < > <= >=
                FilterOperator::Gt | FilterOperator::Lt | FilterOperator::Gte | FilterOperator::Lte => {
                    if matches!(col_type, ColumnType::Text | ColumnType::Boolean) {
                        return Err(AppError::BadRequest(format!(
                            "El operador '{:?}' no es válido para ({})",
                            filter.operator, filter.column
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}
