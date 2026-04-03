use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        project::{CreateProjectDto, UpdateProjectCategoryDto, UpdateProjectDto},
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        milestones,
        prelude::Projects,
        projects::{self, Model as ProjectModel},
    },
    error::AppError,
    state::AppState,
};

#[derive(Clone)]
pub struct ProjectService {
    pub db: Arc<DatabaseConnection>,
}

impl ProjectService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, dto: CreateProjectDto) -> Result<ProjectModel, AppError> {
        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let project = projects::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            description: Set(dto.description),
            category_id: Set(dto.category_id),
            ..Default::default()
        };

        let created_project = project.insert(&txn).await.map_err(AppError::Database)?;

        if let Some(milestones_dto) = dto.milestones {
            if !milestones_dto.is_empty() {
                let new_milestones: Vec<milestones::ActiveModel> = milestones_dto
                    .into_iter()
                    .map(|m_dto| milestones::ActiveModel {
                        project_id: Set(created_project.id),
                        name: Set(m_dto.name),
                        due_date: Set(m_dto.due_date),
                        ..Default::default()
                    })
                    .collect();

                crate::entities::prelude::Milestones::insert_many(new_milestones)
                    .exec(&txn)
                    .await
                    .map_err(AppError::Database)?;
            }
        }

        txn.commit().await.map_err(AppError::Database)?;

        Ok(created_project)
    }

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<ProjectModel, AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado".to_string()))?;

        Ok(project)
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: UpdateProjectDto) -> Result<ProjectModel, AppError> {
        let project = self.get_by_id(user_id, id).await?;
        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let mut active_project: projects::ActiveModel = project.into();
        active_project.name = Set(dto.name);
        active_project.description = Set(dto.description);
        active_project.category_id = Set(dto.category_id);

        let updated_project = active_project.update(&txn).await.map_err(AppError::Database)?;

        if let Some(milestones_dto) = dto.new_milestones {
            if !milestones_dto.is_empty() {
                let new_milestones: Vec<milestones::ActiveModel> = milestones_dto
                    .into_iter()
                    .map(|m_dto| milestones::ActiveModel {
                        project_id: Set(updated_project.id),
                        name: Set(m_dto.name),
                        due_date: Set(m_dto.due_date),
                        ..Default::default()
                    })
                    .collect();

                crate::entities::prelude::Milestones::insert_many(new_milestones)
                    .exec(&txn)
                    .await
                    .map_err(AppError::Database)?;
            }
        }

        txn.commit().await.map_err(AppError::Database)?;

        Ok(updated_project)
    }

    pub async fn update_category(
        &self,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectCategoryDto,
    ) -> Result<ProjectModel, AppError> {
        let project = self.get_by_id(user_id, id).await?;
        let mut active_project: projects::ActiveModel = project.into();

        active_project.category_id = Set(dto.category_id);

        active_project
            .update(self.db.as_ref())
            .await
            .map_err(AppError::Database)
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let project = self.get_by_id(user_id, id).await?;

        projects::ActiveModel {
            id: Set(project.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(
        &self,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<crate::dtos::project::ProjectSummaryDto>, u64), AppError> {
        let mut condition = Condition::all().add(projects::Column::UserId.eq(user_id));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "name" | "description" => {
                    let col = match filter.column.as_str() {
                        "name" => projects::Column::Name,
                        "description" => projects::Column::Description,
                        _ => continue,
                    };
                    let val_str = filter.value.as_str().unwrap_or("").to_string();
                    match filter.operator {
                        FilterOperator::Eq => col.eq(val_str),
                        FilterOperator::Neq => col.ne(val_str),
                        FilterOperator::Contains => col.contains(&val_str),
                        FilterOperator::NotContains => col.not_like(&format!("%{}%", val_str)),
                        _ => continue,
                    }
                }
                "id" | "category_id" => {
                    let val_str = filter.value.as_str().unwrap_or("");
                    if let Ok(uuid_val) = uuid::Uuid::parse_str(val_str) {
                        let col = match filter.column.as_str() {
                            "id" => projects::Column::Id,
                            "category_id" => projects::Column::CategoryId,
                            _ => continue,
                        };
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
                        "created_at" => projects::Column::CreatedAt,
                        "updated_at" => projects::Column::UpdatedAt,
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

        let mut query = Projects::find().filter(condition);

        if let Some(order_col) = payload.order_by {
            let col = match order_col.as_str() {
                "name" => projects::Column::Name,
                "created_at" => projects::Column::CreatedAt,
                _ => projects::Column::CreatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };
            query = query.order_by(col, order);
        } else {
            query = query.order_by_desc(projects::Column::CreatedAt);
        }

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let offset = (payload.page - 1) * payload.limit;

        let models = query
            .left_join(crate::entities::categories::Entity)
            .select_only()
            .column(projects::Column::Id)
            .column(projects::Column::Name)
            .column_as(crate::entities::categories::Column::Name, "category_name")
            .column(projects::Column::CreatedAt)
            .column(projects::Column::UpdatedAt)
            .limit(payload.limit)
            .offset(offset)
            .into_model::<crate::dtos::project::ProjectSummaryDto>()
            .all(self.db.as_ref())
            .await?;

        Ok((models, total_items))
    }
}

impl FromRequestParts<AppState> for ProjectService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(ProjectService::new(state.db.clone()))
    }
}
