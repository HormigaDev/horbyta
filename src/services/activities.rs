use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::Set;
use uuid::Uuid;

use crate::dtos::activity::ActivitySearchField;
use crate::dtos::activity::ActivitySummaryDto;
use crate::dtos::activity::UpdateActivityDto;
use crate::dtos::query::QueryPayload;
use crate::error::AppError;
use crate::utils::search::apply_filters;
use crate::utils::search::apply_ordering;
use crate::utils::search::apply_pagination;
use crate::{
    dtos::activity::{ActivityDetailDto, CreateActivityDto},
    entities::activities,
    entities::prelude::Activities,
};

pub struct ActivityService;

impl ActivityService {
    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateActivityDto,
    ) -> Result<ActivityDetailDto, AppError> {
        let activity = activities::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            ..Default::default()
        };

        let model = activity.insert(db).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ActivityDetailDto, AppError> {
        let activity = Self::find_by_id(db, user_id, id).await?;

        Ok(Self::to_detail_dto(activity))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateActivityDto,
    ) -> Result<ActivityDetailDto, AppError> {
        let activity = Self::find_by_id(db, user_id, id).await?;
        let mut active_activity: activities::ActiveModel = activity.into();

        active_activity.name = Set(dto.name);

        let model = active_activity.update(db).await?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let activity = Self::find_by_id(db, user_id, id).await?;

        activities::ActiveModel {
            id: Set(activity.id),
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
    ) -> Result<(Vec<ActivitySummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(activities::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ActivitySearchField>()
                .ok()
                .map(ActivitySearchField::filter_field)
        })?;

        let query = Activities::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ActivitySearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ActivitySearchField>()
                    .ok()
                    .map(ActivitySearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(
            query
                .select_only()
                .column(activities::Column::Id)
                .column(activities::Column::Name)
                .column(activities::Column::CreatedAt)
                .column(activities::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ActivitySummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<activities::Model, AppError> {
        let activity = Activities::find()
            .filter(activities::Column::Id.eq(id))
            .filter(activities::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Categoría no encontrada".to_string()))?;

        Ok(activity)
    }

    fn to_detail_dto(model: activities::Model) -> ActivityDetailDto {
        ActivityDetailDto {
            id: model.id,
            name: model.name,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
