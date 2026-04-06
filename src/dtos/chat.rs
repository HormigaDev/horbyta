use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::dtos::query::{ColumnType, FilterSchema};

pub fn chat_filter_schema() -> FilterSchema {
    FilterSchema::new()
        .add_column("id", ColumnType::Text)
        .add_column("session_id", ColumnType::Text)
        .add_column("title", ColumnType::Text)
        .add_column("created_at", ColumnType::Date)
        .add_column("updated_at", ColumnType::Date)
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateChatDto {
    #[validate(length(
        min = 1,
        max = 255,
        message = "El título del chat debe tener entre 1 y 255 caracteres."
    ))]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateChatTitleDto {
    #[validate(length(
        min = 1,
        max = 255,
        message = "El título del chat debe tener entre 1 y 255 caracteres."
    ))]
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateChatMessagesDto {
    pub messages: Value,
}

#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct ChatSummaryDto {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatDetailDto {
    pub id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub messages: Value,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
