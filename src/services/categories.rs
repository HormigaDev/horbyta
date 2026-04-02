use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        category::CategoryDto,
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        categories::{self, Model as CategoryModel},
        prelude::Categories,
    },
    error::AppError,
    state::AppState,
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
        let mut condition = Condition::all().add(categories::Column::UserId.eq(user_id));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "name" => {
                    let col = categories::Column::Name;
                    let val_str = filter.value.as_str().unwrap_or("").to_string();
                    match filter.operator {
                        FilterOperator::Eq => col.eq(val_str),
                        FilterOperator::Neq => col.ne(val_str),
                        FilterOperator::Contains => col.contains(&val_str),
                        FilterOperator::NotContains => col.not_like(&format!("%{}%", val_str)),
                        _ => continue,
                    }
                }
                "id" => {
                    let val_str = filter.value.as_str().unwrap_or("");
                    if let Ok(uuid_val) = uuid::Uuid::parse_str(val_str) {
                        let col = categories::Column::Id;
                        match filter.operator {
                            FilterOperator::Eq => col.eq(uuid_val),
                            FilterOperator::Neq => col.ne(uuid_val),
                            _ => continue,
                        }
                    } else {
                        continue;
                    }
                }
                "created_at" | "updated_at" => {
                    let col = match filter.column.as_str() {
                        "created_at" => categories::Column::CreatedAt,
                        "updated_at" => categories::Column::UpdatedAt,
                        _ => continue,
                    };
                    match filter.operator {
                        FilterOperator::Between => {
                            if let Some(arr) = filter.value.as_array() {
                                if arr.len() == 2 {
                                    if let (Some(start_str), Some(end_str)) = (arr[0].as_str(), arr[1].as_str()) {
                                        if let (Ok(start), Ok(end)) = (
                                            chrono::DateTime::parse_from_rfc3339(start_str),
                                            chrono::DateTime::parse_from_rfc3339(end_str),
                                        ) {
                                            col.between(start, end)
                                        } else {
                                            continue;
                                        }
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FilterOperator::Eq => {
                            if let Some(val_str) = filter.value.as_str() {
                                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                    col.eq(date)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FilterOperator::Gt => {
                            if let Some(val_str) = filter.value.as_str() {
                                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                    col.gt(date)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FilterOperator::Lt => {
                            if let Some(val_str) = filter.value.as_str() {
                                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                    col.lt(date)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FilterOperator::Gte => {
                            if let Some(val_str) = filter.value.as_str() {
                                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                    col.gte(date)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FilterOperator::Lte => {
                            if let Some(val_str) = filter.value.as_str() {
                                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(val_str) {
                                    col.lte(date)
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    }
                }
                _ => continue,
            };
            condition = condition.add(filter_cond);
        }

        let mut query = Categories::find().filter(condition);

        if let Some(order_col) = payload.order_by {
            let col = match order_col.as_str() {
                "name" => categories::Column::Name,
                "created_at" => categories::Column::CreatedAt,
                _ => categories::Column::CreatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };
            query = query.order_by(col, order);
        } else {
            query = query.order_by_desc(categories::Column::CreatedAt);
        }

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let offset = (payload.page - 1) * payload.limit;

        let models = query.limit(payload.limit).offset(offset).all(self.db.as_ref()).await?;

        Ok((models, total_items))
    }
}

impl FromRequestParts<AppState> for CategoryService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(CategoryService::new(state.db.clone()))
    }
}
