use sea_orm::FromQueryResult;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        query::{ColumnType, FilterSchema},
        utils::{is_numeric, validate_password},
    },
    entities::clients,
    string_enum,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ClientSearchField {
    Id,
    Name,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl ClientSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::Name => ColumnType::Text,
            Self::Status => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<clients::Column> {
        match self {
            Self::Id => FilterField::Uuid(clients::Column::Id),
            Self::Name => FilterField::Text(clients::Column::Name),
            Self::Status => FilterField::Keyword(clients::Column::Status),
            Self::CreatedAt => FilterField::DateTime(clients::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(clients::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> clients::Column {
        match self {
            Self::Id => clients::Column::Id,
            Self::Name => clients::Column::Name,
            Self::Status => clients::Column::Status,
            Self::CreatedAt => clients::Column::CreatedAt,
            Self::UpdatedAt => clients::Column::UpdatedAt,
        }
    }
}

pub fn client_filter_schema() -> FilterSchema {
    ClientSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
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

    pub status: ClientStatus,

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

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateClientStatusDto {
    pub status: ClientStatus,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientSummary {
    pub id: Uuid,
    pub name: String,
    pub status: ClientStatus,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ClientAuthDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub email: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct ClientAuthStatusDto {
    pub method: ClientAuthMethod,
    pub email: Option<String>,
    pub magic_client_token_exp: Option<i64>,
}

#[derive(Serialize)]
pub struct ClientMagicLinkDto {
    pub magic_client_token: String,
    pub magic_client_token_exp: i64,
}

#[derive(Deserialize, Validate)]
pub struct SaveClientCredentialsDto {
    #[validate(email(message = "Email de autenticación para el cliente inválido"))]
    pub email: String,

    #[validate(custom(function = "validate_password"))]
    pub password: String,
}

string_enum!(
    pub enum ClientAuthMethod {
        None,
        Credentials,
        MagicToken,
    }
);

string_enum!(
    pub enum ClientStatus {
        Active,
        Inactive,
        Blocked,
    }
);
