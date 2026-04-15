use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    dtos::{
        chat::{
            ChatDetailDto, ChatSearchField, ChatSummaryDto, CreateChatDto, UpdateChatMessagesDto, UpdateChatTitleDto,
        },
        query::QueryPayload,
    },
    entities::{
        conversation_history::{self, Model as ConversationModel},
        prelude::ConversationHistory,
    },
    error::AppError,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

const DEFAULT_CHAT_TITLE: &str = "Nuevo chat";

pub struct ChatService;

impl ChatService {
    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateChatDto,
    ) -> Result<ChatDetailDto, AppError> {
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
        .insert(db)
        .await
        .map_err(AppError::Database)?;

        Ok(Self::to_detail(created))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ChatDetailDto, AppError> {
        let model = Self::get_model(db, user_id, id).await?;
        Ok(Self::to_detail(model))
    }

    pub async fn update_title(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChatTitleDto,
    ) -> Result<ChatDetailDto, AppError> {
        let model = Self::get_model(db, user_id, id).await?;
        let mut active: conversation_history::ActiveModel = model.into();
        active.title = Set(dto.title.trim().to_string());

        let updated = active.update(db).await.map_err(AppError::Database)?;
        Ok(Self::to_detail(updated))
    }

    pub async fn update_messages(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChatMessagesDto,
    ) -> Result<ChatDetailDto, AppError> {
        let model = Self::get_model(db, user_id, id).await?;
        let mut active: conversation_history::ActiveModel = model.into();
        active.messages = Set(dto.messages);

        let updated = active.update(db).await.map_err(AppError::Database)?;
        Ok(Self::to_detail(updated))
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let model = Self::get_model(db, user_id, id).await?;

        conversation_history::ActiveModel {
            id: Set(model.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ChatSummaryDto>, u64), AppError> {
        let base_condition = Condition::all()
            .add(conversation_history::Column::UserId.eq(user_id))
            .add(conversation_history::Column::Messages.ne(json!([])));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ChatSearchField>()
                .ok()
                .map(ChatSearchField::filter_field)
        })?;

        let query = ConversationHistory::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ChatSearchField::UpdatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ChatSearchField>()
                    .ok()
                    .map(ChatSearchField::order_column)
            },
        )?;

        let total = query.clone().count(db).await.map_err(AppError::Database)?;

        let rows = apply_pagination(
            query
                .select_only()
                .column(conversation_history::Column::Id)
                .column(conversation_history::Column::SessionId)
                .column(conversation_history::Column::Title)
                .column(conversation_history::Column::CreatedAt)
                .column(conversation_history::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ChatSummaryDto>()
        .all(db)
        .await
        .map_err(AppError::Database)?;

        Ok((rows, total))
    }

    async fn get_model(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ConversationModel, AppError> {
        let model = ConversationHistory::find()
            .filter(conversation_history::Column::Id.eq(id))
            .filter(conversation_history::Column::UserId.eq(user_id))
            .one(db)
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
