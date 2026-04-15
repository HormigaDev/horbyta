use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        query::QueryPayload,
        tag::{CreateTagDto, TagDetailDto, TagSearchField, TagSummaryDto, UpdateTagDto},
    },
    entities::{
        prelude::Tags,
        tags::{self, Model as TagModel},
    },
    error::AppError,
    state::AppState,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

#[derive(Clone)]
pub struct TagService {
    pub db: Arc<DatabaseConnection>,
}

impl TagService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, dto: CreateTagDto) -> Result<TagDetailDto, AppError> {
        let tag = tags::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            color: Set(dto.color.to_string()),
            ..Default::default()
        };

        let model = tag.insert(self.db.as_ref()).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<TagDetailDto, AppError> {
        let tag = self.find_by_id(user_id, id).await?;

        Ok(Self::to_detail_dto(tag))
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: UpdateTagDto) -> Result<TagDetailDto, AppError> {
        let tag = self.find_by_id(user_id, id).await?;
        let mut active_tag: tags::ActiveModel = tag.into();

        active_tag.name = Set(dto.name);
        active_tag.color = Set(dto.color.to_string());

        let model = active_tag.update(self.db.as_ref()).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let tag = self.find_by_id(user_id, id).await?;

        tags::ActiveModel {
            id: Set(tag.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<TagSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(tags::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column.parse::<TagSearchField>().ok().map(TagSearchField::filter_field)
        })?;

        let query = Tags::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            TagSearchField::CreatedAt.order_column(),
            false,
            |column| column.parse::<TagSearchField>().ok().map(TagSearchField::order_column),
        )?;

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let models = apply_pagination(
            query
                .select_only()
                .column(tags::Column::Id)
                .column(tags::Column::Name)
                .column(tags::Column::Color),
            &payload,
        )
        .into_model::<TagSummaryDto>()
        .all(self.db.as_ref())
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<TagModel, AppError> {
        let tag = Tags::find()
            .filter(tags::Column::Id.eq(id))
            .filter(tags::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Etiqueta no encontrada".to_string()))?;

        Ok(tag)
    }

    fn to_detail_dto(model: TagModel) -> TagDetailDto {
        TagDetailDto {
            id: model.id,
            name: model.name,
            color: model.color,
        }
    }
}

impl FromRequestParts<AppState> for TagService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(TagService::new(state.db.clone()))
    }
}
