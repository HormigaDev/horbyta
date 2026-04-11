use sea_orm::FromQueryResult;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::dtos::query::{ColumnType, FilterSchema};
use crate::dtos::utils::is_numeric;

pub fn client_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("name", ColumnType::Text)
        .add_column("status", ColumnType::Text)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

pub fn is_client_status(s: &str) -> Result<(), ValidationError> {
    let allowed_client_status = ["active", "inactive", "blocked"];
    if allowed_client_status.contains(&s) {
        Ok(())
    } else {
        Err(ValidationError::new("Estado del cliente inválido"))
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct CreateClientDto {
    #[validate(length(
        min = 1,
        max = 255,
        message = "El nombre del cliente debe tener entre 1 y 255 caracteres"
    ))]
    pub name: String,

    #[validate(
        email(message = "Correo electrónico del cliente inválido"),
        length(max = 255, message = "Correo electrónico del cliente inválido")
    )]
    pub email: Option<String>,

    #[validate(
        custom(function = "is_numeric"),
        length(max = 20, message = "Número de teléfono del cliente inválido")
    )]
    pub phone: Option<String>,

    #[validate(length(max = 512, message = "La dirección del cliente no puede superar los 512 caracteres"))]
    pub address_line: Option<String>,

    #[validate(length(
        max = 255,
        message = "El nombre de la compañía del cliente no puede superar los 255 caracteres"
    ))]
    pub company: Option<String>,

    #[validate(length(max = 5000, message = "Las notas del cliente no pueden superar los 5000 caracteres"))]
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone)]
pub struct UpdateClientDto {
    #[validate(length(
        min = 1,
        max = 255,
        message = "El nombre del cliente debe tener entre 1 y 255 caracteres"
    ))]
    pub name: String,

    #[validate(
        email(message = "Correo electrónico del cliente inválido"),
        length(max = 255, message = "Correo electrónico del cliente inválido")
    )]
    pub email: Option<String>,

    #[validate(custom(function = "is_client_status"))]
    pub status: String,

    #[validate(
        custom(function = "is_numeric"),
        length(max = 20, message = "Número de teléfono del cliente inválido")
    )]
    pub phone: Option<String>,

    #[validate(length(max = 512, message = "La dirección del cliente no puede superar los 512 caracteres"))]
    pub address_line: Option<String>,

    #[validate(length(
        max = 255,
        message = "El nombre de la compañía del cliente no puede superar los 255 caracteres"
    ))]
    pub company: Option<String>,

    #[validate(length(max = 5000, message = "Las notas del cliente no pueden superar los 5000 caracteres"))]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateClientStatusDto {
    #[validate(custom(function = "is_client_status"))]
    pub status: String,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientSummary {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
