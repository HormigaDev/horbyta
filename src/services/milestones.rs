use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        milestone::MilestoneDto,
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        milestones::{self, Model as MilestoneModel},
        prelude::{Milestones, Projects},
        projects,
    },
    error::AppError,
    state::AppState,
};

#[derive(Clone)]
pub struct MilestoneService {
    pub db: Arc<DatabaseConnection>,
}

impl MilestoneService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    async fn check_project_ownership(&self, user_id: Uuid, project_id: Uuid) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?;

        if project.is_none() {
            return Err(AppError::NotFound(
                "Proyecto no encontrado o no tienes permiso".to_string(),
            ));
        }
        Ok(())
    }

    async fn check_milestone_ownership(&self, user_id: Uuid, milestone_id: Uuid) -> Result<MilestoneModel, AppError> {
        // Find milestone and join project to verify user
        let milestone_opt = Milestones::find()
            .filter(milestones::Column::Id.eq(milestone_id))
            .find_also_related(Projects)
            .one(self.db.as_ref())
            .await?;

        match milestone_opt {
            Some((milestone, Some(project))) => {
                if project.user_id != user_id {
                    return Err(AppError::NotFound("Hito no encontrado o no tienes permiso".to_string()));
                }
                Ok(milestone)
            }
            _ => Err(AppError::NotFound("Hito no encontrado".to_string())),
        }
    }

    pub async fn create(&self, user_id: Uuid, dto: MilestoneDto) -> Result<MilestoneModel, AppError> {
        self.check_project_ownership(user_id, dto.project_id).await?;

        let milestone = milestones::ActiveModel {
            project_id: Set(dto.project_id),
            name: Set(dto.name),
            due_date: Set(dto.due_date),
            ..Default::default()
        };

        milestone.insert(self.db.as_ref()).await.map_err(AppError::Database)
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: MilestoneDto) -> Result<MilestoneModel, AppError> {
        let milestone = self.check_milestone_ownership(user_id, id).await?;

        if milestone.project_id != dto.project_id {
            self.check_project_ownership(user_id, dto.project_id).await?;
        }

        let mut active_milestone: milestones::ActiveModel = milestone.into();
        active_milestone.project_id = Set(dto.project_id);
        active_milestone.name = Set(dto.name);
        active_milestone.due_date = Set(dto.due_date);

        active_milestone
            .update(self.db.as_ref())
            .await
            .map_err(AppError::Database)
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let milestone = self.check_milestone_ownership(user_id, id).await?;

        milestones::ActiveModel {
            id: Set(milestone.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<MilestoneModel>, u64), AppError> {
        // We only return milestones for projects that belong to the user
        let project_ids: Vec<Uuid> = Projects::find()
            .filter(projects::Column::UserId.eq(user_id))
            .select_only()
            .column(projects::Column::Id)
            .into_tuple()
            .all(self.db.as_ref())
            .await?;

        let mut condition = Condition::all().add(milestones::Column::ProjectId.is_in(project_ids));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "name" => {
                    let col = milestones::Column::Name;
                    let val_str = filter.value.as_str().unwrap_or("").to_string();
                    match filter.operator {
                        FilterOperator::Eq => col.eq(val_str),
                        FilterOperator::Neq => col.ne(val_str),
                        FilterOperator::Contains => col.contains(&val_str),
                        FilterOperator::NotContains => col.not_like(&format!("%{}%", val_str)),
                        _ => continue,
                    }
                }
                "id" | "project_id" => {
                    let val_str = filter.value.as_str().unwrap_or("");
                    if let Ok(uuid_val) = uuid::Uuid::parse_str(val_str) {
                        let col = match filter.column.as_str() {
                            "id" => milestones::Column::Id,
                            "project_id" => milestones::Column::ProjectId,
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
                "created_at" | "updated_at" | "due_date" => {
                    let col = match filter.column.as_str() {
                        "created_at" => milestones::Column::CreatedAt,
                        "updated_at" => milestones::Column::UpdatedAt,
                        "due_date" => milestones::Column::DueDate,
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

        let mut query = Milestones::find().filter(condition);

        if let Some(order_col) = payload.order_by {
            let col = match order_col.as_str() {
                "name" => milestones::Column::Name,
                "due_date" => milestones::Column::DueDate,
                "created_at" => milestones::Column::CreatedAt,
                _ => milestones::Column::CreatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };
            query = query.order_by(col, order);
        } else {
            query = query.order_by_desc(milestones::Column::CreatedAt);
        }

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let offset = (payload.page - 1) * payload.limit;

        let models = query.limit(payload.limit).offset(offset).all(self.db.as_ref()).await?;

        Ok((models, total_items))
    }
}

impl FromRequestParts<AppState> for MilestoneService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(MilestoneService::new(state.db.clone()))
    }
}
