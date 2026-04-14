use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        category::{CategoryDto, CategorySearchField},
        query::QueryPayload,
    },
    entities::{
        categories::{self, Model as CategoryModel},
        prelude::Categories,
    },
    error::AppError,
    state::AppState,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

#[derive(Clone)]
pub struct CategoryService {
    pub db: Arc<DatabaseConnection>,
}

impl CategoryService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, dto: CategoryDto) -> Result<CategoryModel, AppError> {
        let category = categories::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            ..Default::default()
        };

        category.insert(self.db.as_ref()).await.map_err(AppError::Database)
    }

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<CategoryModel, AppError> {
        let category = Categories::find()
            .filter(categories::Column::Id.eq(id))
            .filter(categories::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Categoría no encontrada".to_string()))?;

        Ok(category)
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: CategoryDto) -> Result<CategoryModel, AppError> {
        let category = self.get_by_id(user_id, id).await?;
        let mut active_category: categories::ActiveModel = category.into();

        active_category.name = Set(dto.name);

        active_category
            .update(self.db.as_ref())
            .await
            .map_err(AppError::Database)
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let category = self.get_by_id(user_id, id).await?;

        categories::ActiveModel {
            id: Set(category.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<CategoryModel>, u64), AppError> {
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

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let models = apply_pagination(query, &payload).all(self.db.as_ref()).await?;

        Ok((models, total_items))
    }
}

impl FromRequestParts<AppState> for CategoryService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(CategoryService::new(state.db.clone()))
    }
}
