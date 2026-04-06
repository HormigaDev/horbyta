use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        chat::{ChatDetailDto, ChatSummaryDto, CreateChatDto, UpdateChatTitleDto},
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        conversation_history::{self, Model as ConversationModel},
        prelude::ConversationHistory,
    },
    error::AppError,
    state::AppState,
};

const DEFAULT_CHAT_TITLE: &str = "Nuevo chat";

#[derive(Clone)]
pub struct ChatService {
    pub db: Arc<DatabaseConnection>,
}

impl ChatService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, dto: CreateChatDto) -> Result<ChatDetailDto, AppError> {
        let title = dto
            .title
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_CHAT_TITLE.to_string());

        let created = conversation_history::ActiveModel {
            user_id: Set(user_id),
            session_id: Set(Uuid::now_v7()),
            title: Set(title),
            messages: Set(json!([])),
            ..Default::default()
        }
        .insert(self.db.as_ref())
        .await
        .map_err(AppError::Database)?;

        Ok(Self::to_detail(created))
    }

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<ChatDetailDto, AppError> {
        let model = self.get_model(user_id, id).await?;
        Ok(Self::to_detail(model))
    }

    pub async fn update_title(
        &self,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChatTitleDto,
    ) -> Result<ChatDetailDto, AppError> {
        let model = self.get_model(user_id, id).await?;
        let mut active: conversation_history::ActiveModel = model.into();
        active.title = Set(dto.title.trim().to_string());

        let updated = active.update(self.db.as_ref()).await.map_err(AppError::Database)?;
        Ok(Self::to_detail(updated))
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let model = self.get_model(user_id, id).await?;

        conversation_history::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<ChatSummaryDto>, u64), AppError> {
        let mut condition = Condition::all().add(conversation_history::Column::UserId.eq(user_id));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "title" => {
                    let value = filter.value.as_str().unwrap_or("").to_string();
                    match filter.operator {
                        FilterOperator::Eq => conversation_history::Column::Title.eq(value),
                        FilterOperator::Neq => conversation_history::Column::Title.ne(value),
                        FilterOperator::Contains => conversation_history::Column::Title.contains(&value),
                        FilterOperator::NotContains => {
                            conversation_history::Column::Title.not_like(&format!("%{}%", value))
                        }
                        _ => continue,
                    }
                }
                "id" | "session_id" => {
                    let value = filter.value.as_str().unwrap_or("");
                    let uuid_value = match Uuid::parse_str(value) {
                        Ok(parsed) => parsed,
                        Err(_) => continue,
                    };

                    let column = if filter.column == "id" {
                        conversation_history::Column::Id
                    } else {
                        conversation_history::Column::SessionId
                    };

                    match filter.operator {
                        FilterOperator::Eq => column.eq(uuid_value),
                        FilterOperator::Neq => column.ne(uuid_value),
                        _ => continue,
                    }
                }
                "created_at" | "updated_at" => {
                    let column = if filter.column == "created_at" {
                        conversation_history::Column::CreatedAt
                    } else {
                        conversation_history::Column::UpdatedAt
                    };

                    match filter.operator {
                        FilterOperator::Between => {
                            let values = match filter.value.as_array() {
                                Some(arr) if arr.len() == 2 => arr,
                                _ => continue,
                            };

                            let start = values[0]
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            let end = values[1]
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());

                            let (start, end) = match (start, end) {
                                (Some(start_date), Some(end_date)) => (start_date, end_date),
                                _ => continue,
                            };

                            column.between(start, end)
                        }
                        FilterOperator::Eq => {
                            let parsed = filter
                                .value
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            match parsed {
                                Some(date) => column.eq(date),
                                None => continue,
                            }
                        }
                        FilterOperator::Gt => {
                            let parsed = filter
                                .value
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            match parsed {
                                Some(date) => column.gt(date),
                                None => continue,
                            }
                        }
                        FilterOperator::Lt => {
                            let parsed = filter
                                .value
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            match parsed {
                                Some(date) => column.lt(date),
                                None => continue,
                            }
                        }
                        FilterOperator::Gte => {
                            let parsed = filter
                                .value
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            match parsed {
                                Some(date) => column.gte(date),
                                None => continue,
                            }
                        }
                        FilterOperator::Lte => {
                            let parsed = filter
                                .value
                                .as_str()
                                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                            match parsed {
                                Some(date) => column.lte(date),
                                None => continue,
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            };

            condition = condition.add(filter_cond);
        }

        let mut query = ConversationHistory::find().filter(condition);

        if let Some(order_column) = payload.order_by {
            let column = match order_column.as_str() {
                "title" => conversation_history::Column::Title,
                "created_at" => conversation_history::Column::CreatedAt,
                "updated_at" => conversation_history::Column::UpdatedAt,
                _ => conversation_history::Column::UpdatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };

            query = query.order_by(column, order);
        } else {
            query = query.order_by_desc(conversation_history::Column::UpdatedAt);
        }

        let total = query.clone().count(self.db.as_ref()).await.map_err(AppError::Database)?;
        let offset = (payload.page - 1) * payload.limit;

        let rows = query
            .select_only()
            .column(conversation_history::Column::Id)
            .column(conversation_history::Column::SessionId)
            .column(conversation_history::Column::Title)
            .column(conversation_history::Column::CreatedAt)
            .column(conversation_history::Column::UpdatedAt)
            .limit(payload.limit)
            .offset(offset)
            .into_model::<ChatSummaryDto>()
            .all(self.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        Ok((rows, total))
    }

    async fn get_model(&self, user_id: Uuid, id: Uuid) -> Result<ConversationModel, AppError> {
        let model = ConversationHistory::find()
            .filter(conversation_history::Column::Id.eq(id))
            .filter(conversation_history::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("Chat no encontrado".to_string()))?;

        Ok(model)
    }

    fn to_detail(model: ConversationModel) -> ChatDetailDto {
        let messages = match model.messages {
            Value::Array(_) => model.messages,
            _ => json!([]),
        };

        ChatDetailDto {
            id: model.id,
            session_id: model.session_id,
            title: model.title,
            messages,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl FromRequestParts<AppState> for ChatService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(ChatService::new(state.db.clone()))
    }
}