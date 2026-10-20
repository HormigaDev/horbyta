use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use uuid::Uuid;

use crate::error_codes as codes;
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
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub struct TagService;

impl TagService {
    pub async fn create(db: &impl ConnectionTrait, user_id: Uuid, dto: CreateTagDto) -> Result<TagDetailDto, AppError> {
        let tag = tags::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            color: Set(dto.color.to_string()),
            ..Default::default()
        };

        let model = tag.insert(db).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TagDetailDto, AppError> {
        let tag = Self::find_by_id(db, user_id, id).await?;

        Ok(Self::to_detail_dto(tag))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateTagDto,
    ) -> Result<TagDetailDto, AppError> {
        let tag = Self::find_by_id(db, user_id, id).await?;
        let mut active_tag: tags::ActiveModel = tag.into();

        active_tag.name = Set(dto.name);
        active_tag.color = Set(dto.color.to_string());

        let model = active_tag.update(db).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let tag = Self::find_by_id(db, user_id, id).await?;

        tags::ActiveModel {
            id: Set(tag.id),
            ..Default::default()
        }
        .delete(db)
        .await?;

        Ok(())
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<TagSummaryDto>, u64), AppError> {
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

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(
            query
                .select_only()
                .column(tags::Column::Id)
                .column(tags::Column::Name)
                .column(tags::Column::Color),
            &payload,
        )
        .into_model::<TagSummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TagModel, AppError> {
        let tag = Tags::find()
            .filter(tags::Column::Id.eq(id))
            .filter(tags::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::TAG_NOT_FOUND.into()))?;

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
