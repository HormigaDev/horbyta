use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::IntoEnumIterator;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::conversation_history,
    utils::search::FilterField,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ChatSearchField {
    Id,
    SessionId,
    Title,
    CreatedAt,
    UpdatedAt,
}

impl ChatSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::SessionId => "session_id",
            Self::Title => "title",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Text,
            Self::SessionId => ColumnType::Text,
            Self::Title => ColumnType::Text,
            Self::CreatedAt => ColumnType::Date,
            Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<conversation_history::Column> {
        match self {
            Self::Id => FilterField::Uuid(conversation_history::Column::Id),
            Self::SessionId => FilterField::Uuid(conversation_history::Column::SessionId),
            Self::Title => FilterField::Text(conversation_history::Column::Title),
            Self::CreatedAt => FilterField::DateTime(conversation_history::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(conversation_history::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> conversation_history::Column {
        match self {
            Self::Id => conversation_history::Column::Id,
            Self::SessionId => conversation_history::Column::SessionId,
            Self::Title => conversation_history::Column::Title,
            Self::CreatedAt => conversation_history::Column::CreatedAt,
            Self::UpdatedAt => conversation_history::Column::UpdatedAt,
        }
    }
}

pub fn chat_filter_schema() -> FilterSchema {
    ChatSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateChatDto {
    #[validate(length(min = 1, max = 255, message = "chat title must be between 1 and 255 characters"))]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateChatTitleDto {
    #[validate(length(min = 1, max = 255, message = "chat title must be between 1 and 255 characters"))]
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
