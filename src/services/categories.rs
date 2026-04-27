use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use uuid::Uuid;

use crate::{
    dtos::{
        category::{CategoryDetailDto, CategorySearchField, CategorySummaryDto, CreateCategoryDto, UpdateCategoryDto},
        query::QueryPayload,
    },
    entities::{
        categories::{self, Model as CategoryModel},
        prelude::Categories,
    },
    error::AppError,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub struct CategoryService;

impl CategoryService {
    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateCategoryDto,
    ) -> Result<CategoryDetailDto, AppError> {
        let category = categories::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            ..Default::default()
        };

        let model = category.insert(db).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<CategoryDetailDto, AppError> {
        let category = Self::find_by_id(db, user_id, id).await?;

        Ok(Self::to_detail_dto(category))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateCategoryDto,
    ) -> Result<CategoryDetailDto, AppError> {
        let category = Self::find_by_id(db, user_id, id).await?;
        let mut active_category: categories::ActiveModel = category.into();

        active_category.name = Set(dto.name);

        let model = active_category.update(db).await.map_err(AppError::Database)?;

        Ok(Self::to_detail_dto(model))
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let category = Self::find_by_id(db, user_id, id).await?;

        categories::ActiveModel {
            id: Set(category.id),
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
    ) -> Result<(Vec<CategorySummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(categories::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<CategorySearchField>()
                .ok()
                .map(CategorySearchField::filter_field)
        })?;

        let query = Categories::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            CategorySearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<CategorySearchField>()
                    .ok()
                    .map(CategorySearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(
            query
                .select_only()
                .column(categories::Column::Id)
                .column(categories::Column::Name),
            &payload,
        )
        .into_model::<CategorySummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<CategoryModel, AppError> {
        let category = Categories::find()
            .filter(categories::Column::Id.eq(id))
            .filter(categories::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("category not found".to_string()))?;

        Ok(category)
    }

    fn to_detail_dto(model: CategoryModel) -> CategoryDetailDto {
        CategoryDetailDto {
            id: model.id,
            name: model.name,
        }
    }
}
