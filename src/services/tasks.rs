use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

use crate::{
    dtos::{
        query::{FilterOperator, QueryPayload},
        task::{
            CreateTaskDto, LookupOptionDto, MilestoneLookupDto, TaskDetailDto, TaskSummaryDto, TaskViewDto,
            UpdateTaskDto,
        },
    },
    entities::{
        milestones,
        prelude::{Milestones, Projects, Tags, TaskTags, Tasks},
        projects, tags,
        task_tags::{self},
        tasks::{self, Model as TaskModel},
    },
    error::AppError,
    state::AppState,
};

#[derive(Clone)]
pub struct TaskService {
    pub db: Arc<DatabaseConnection>,
}

impl TaskService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    async fn get_task_model(&self, user_id: Uuid, id: Uuid) -> Result<TaskModel, AppError> {
        let task = Tasks::find()
            .filter(tasks::Column::Id.eq(id))
            .filter(tasks::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Tarea no encontrada".to_string()))?;

        Ok(task)
    }

    async fn get_tag_ids_for_task(&self, task_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let tag_ids: Vec<Uuid> = TaskTags::find()
            .filter(task_tags::Column::TaskId.eq(task_id))
            .select_only()
            .column(task_tags::Column::TagId)
            .into_tuple()
            .all(self.db.as_ref())
            .await?;

        Ok(tag_ids)
    }

    async fn to_task_detail(&self, task: TaskModel) -> Result<TaskDetailDto, AppError> {
        let tag_ids = self.get_tag_ids_for_task(task.id).await?;

        let tags = if tag_ids.is_empty() {
            Vec::new()
        } else {
            Tags::find()
                .filter(tags::Column::UserId.eq(task.user_id))
                .filter(tags::Column::Id.is_in(tag_ids.clone()))
                .order_by_asc(tags::Column::Name)
                .all(self.db.as_ref())
                .await?
                .into_iter()
                .map(|tag| LookupOptionDto {
                    id: tag.id,
                    name: tag.name,
                })
                .collect::<Vec<LookupOptionDto>>()
        };

        let project_name = if let Some(project_id) = task.project_id {
            Projects::find()
                .filter(projects::Column::Id.eq(project_id))
                .filter(projects::Column::UserId.eq(task.user_id))
                .one(self.db.as_ref())
                .await?
                .map(|project| project.name)
        } else {
            None
        };

        let milestone_name = if let Some(milestone_id) = task.milestone_id {
            Milestones::find()
                .filter(milestones::Column::Id.eq(milestone_id))
                .one(self.db.as_ref())
                .await?
                .map(|milestone| milestone.name)
        } else {
            None
        };

        Ok(TaskDetailDto {
            id: task.id,
            user_id: task.user_id,
            project_id: task.project_id,
            parent_id: task.parent_id,
            milestone_id: task.milestone_id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            complexity: task.complexity,
            block_parent: task.block_parent,
            due_date: task.due_date,
            project_name,
            milestone_name,
            created_at: task.created_at,
            updated_at: task.updated_at,
            tag_ids,
            tags,
        })
    }

    async fn ensure_project_ownership(&self, user_id: Uuid, project_id: Option<Uuid>) -> Result<(), AppError> {
        if let Some(pid) = project_id {
            let project = Projects::find()
                .filter(projects::Column::Id.eq(pid))
                .filter(projects::Column::UserId.eq(user_id))
                .one(self.db.as_ref())
                .await?;

            if project.is_none() {
                return Err(AppError::BadRequest("Proyecto inválido o sin permisos".to_string()));
            }
        }

        Ok(())
    }

    async fn ensure_parent_ownership(&self, user_id: Uuid, parent_id: Option<Uuid>) -> Result<(), AppError> {
        if let Some(pid) = parent_id {
            let parent = Tasks::find()
                .filter(tasks::Column::Id.eq(pid))
                .filter(tasks::Column::UserId.eq(user_id))
                .one(self.db.as_ref())
                .await?;

            if parent.is_none() {
                return Err(AppError::BadRequest("Tarea padre inválida o sin permisos".to_string()));
            }
        }

        Ok(())
    }

    async fn ensure_milestone_ownership(
        &self,
        user_id: Uuid,
        milestone_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(mid) = milestone_id {
            let milestone_join = Milestones::find()
                .filter(milestones::Column::Id.eq(mid))
                .find_also_related(Projects)
                .one(self.db.as_ref())
                .await?;

            match milestone_join {
                Some((milestone, Some(project))) => {
                    if project.user_id != user_id {
                        return Err(AppError::BadRequest("Milestone inválido o sin permisos".to_string()));
                    }

                    if let Some(pid) = project_id {
                        if milestone.project_id != pid {
                            return Err(AppError::BadRequest(
                                "El milestone no pertenece al proyecto seleccionado".to_string(),
                            ));
                        }
                    }
                }
                _ => {
                    return Err(AppError::BadRequest("Milestone inválido".to_string()));
                }
            }
        }

        Ok(())
    }

    async fn validate_and_dedup_tags(&self, user_id: Uuid, tag_ids: &[Uuid]) -> Result<Vec<Uuid>, AppError> {
        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        let deduped_ids: Vec<Uuid> = tag_ids.iter().copied().collect::<HashSet<Uuid>>().into_iter().collect();

        let found_count = Tags::find()
            .filter(tags::Column::UserId.eq(user_id))
            .filter(tags::Column::Id.is_in(deduped_ids.clone()))
            .count(self.db.as_ref())
            .await?;

        if found_count != deduped_ids.len() as u64 {
            return Err(AppError::BadRequest("Una o más etiquetas no son válidas".to_string()));
        }

        Ok(deduped_ids)
    }

    pub async fn create(&self, user_id: Uuid, dto: CreateTaskDto) -> Result<TaskDetailDto, AppError> {
        self.ensure_project_ownership(user_id, dto.project_id).await?;
        self.ensure_parent_ownership(user_id, dto.parent_id).await?;
        self.ensure_milestone_ownership(user_id, dto.milestone_id, dto.project_id)
            .await?;
        let safe_tag_ids = self.validate_and_dedup_tags(user_id, &dto.tag_ids).await?;

        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let task = tasks::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            parent_id: Set(dto.parent_id),
            milestone_id: Set(dto.milestone_id),
            title: Set(dto.title),
            description: Set(dto.description),
            status: Set(dto.status),
            priority: Set(dto.priority),
            complexity: Set(dto.complexity),
            block_parent: Set(dto.block_parent),
            due_date: Set(dto.due_date),
            ..Default::default()
        };

        let created_task = task.insert(&txn).await.map_err(AppError::Database)?;

        if !safe_tag_ids.is_empty() {
            let links: Vec<task_tags::ActiveModel> = safe_tag_ids
                .iter()
                .copied()
                .map(|tag_id| task_tags::ActiveModel {
                    task_id: Set(created_task.id),
                    tag_id: Set(tag_id),
                })
                .collect();

            TaskTags::insert_many(links)
                .exec(&txn)
                .await
                .map_err(AppError::Database)?;
        }

        txn.commit().await.map_err(AppError::Database)?;

        self.get_by_id(user_id, created_task.id).await
    }

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<TaskDetailDto, AppError> {
        let task = self.get_task_model(user_id, id).await?;
        self.to_task_detail(task).await
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: UpdateTaskDto) -> Result<TaskDetailDto, AppError> {
        if dto.parent_id == Some(id) {
            return Err(AppError::BadRequest(
                "Una tarea no puede ser padre de sí misma".to_string(),
            ));
        }

        let task = self.get_task_model(user_id, id).await?;
        self.ensure_project_ownership(user_id, dto.project_id).await?;
        self.ensure_parent_ownership(user_id, dto.parent_id).await?;
        self.ensure_milestone_ownership(user_id, dto.milestone_id, dto.project_id)
            .await?;
        let safe_tag_ids = self.validate_and_dedup_tags(user_id, &dto.tag_ids).await?;

        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let mut active_task: tasks::ActiveModel = task.into();
        active_task.project_id = Set(dto.project_id);
        active_task.parent_id = Set(dto.parent_id);
        active_task.milestone_id = Set(dto.milestone_id);
        active_task.title = Set(dto.title);
        active_task.description = Set(dto.description);
        active_task.status = Set(dto.status);
        active_task.priority = Set(dto.priority);
        active_task.complexity = Set(dto.complexity);
        active_task.block_parent = Set(dto.block_parent);
        active_task.due_date = Set(dto.due_date);

        let updated_task = active_task.update(&txn).await.map_err(AppError::Database)?;

        TaskTags::delete_many()
            .filter(task_tags::Column::TaskId.eq(updated_task.id))
            .exec(&txn)
            .await
            .map_err(AppError::Database)?;

        if !safe_tag_ids.is_empty() {
            let links: Vec<task_tags::ActiveModel> = safe_tag_ids
                .iter()
                .copied()
                .map(|tag_id| task_tags::ActiveModel {
                    task_id: Set(updated_task.id),
                    tag_id: Set(tag_id),
                })
                .collect();

            TaskTags::insert_many(links)
                .exec(&txn)
                .await
                .map_err(AppError::Database)?;
        }

        txn.commit().await.map_err(AppError::Database)?;

        self.get_by_id(user_id, updated_task.id).await
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let task = self.get_task_model(user_id, id).await?;

        tasks::ActiveModel {
            id: Set(task.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<TaskSummaryDto>, u64), AppError> {
        let mut condition = Condition::all().add(tasks::Column::UserId.eq(user_id));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "title" | "description" | "status" => {
                    let col = match filter.column.as_str() {
                        "title" => tasks::Column::Title,
                        "description" => tasks::Column::Description,
                        "status" => tasks::Column::Status,
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
                "id" | "project_id" | "parent_id" | "milestone_id" => {
                    let val_str = filter.value.as_str().unwrap_or("");
                    if let Ok(uuid_val) = Uuid::parse_str(val_str) {
                        let col = match filter.column.as_str() {
                            "id" => tasks::Column::Id,
                            "project_id" => tasks::Column::ProjectId,
                            "parent_id" => tasks::Column::ParentId,
                            "milestone_id" => tasks::Column::MilestoneId,
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
                "priority" | "complexity" => {
                    let maybe_num = filter
                        .value
                        .as_i64()
                        .or_else(|| filter.value.as_str().and_then(|s| s.parse::<i64>().ok()));

                    let num = if let Some(v) = maybe_num {
                        v as i16
                    } else {
                        continue;
                    };

                    let col = match filter.column.as_str() {
                        "priority" => tasks::Column::Priority,
                        "complexity" => tasks::Column::Complexity,
                        _ => continue,
                    };

                    match filter.operator {
                        FilterOperator::Eq => col.eq(num),
                        FilterOperator::Neq => col.ne(num),
                        FilterOperator::Gt => col.gt(num),
                        FilterOperator::Lt => col.lt(num),
                        FilterOperator::Gte => col.gte(num),
                        FilterOperator::Lte => col.lte(num),
                        _ => continue,
                    }
                }
                "block_parent" => {
                    let bool_val = if let Some(v) = filter.value.as_bool() {
                        v
                    } else {
                        continue;
                    };

                    match filter.operator {
                        FilterOperator::Eq => tasks::Column::BlockParent.eq(bool_val),
                        FilterOperator::Neq => tasks::Column::BlockParent.ne(bool_val),
                        _ => continue,
                    }
                }
                "due_date" | "created_at" | "updated_at" => {
                    let col = match filter.column.as_str() {
                        "due_date" => tasks::Column::DueDate,
                        "created_at" => tasks::Column::CreatedAt,
                        "updated_at" => tasks::Column::UpdatedAt,
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

        let mut query = Tasks::find().filter(condition);

        if let Some(order_col) = payload.order_by {
            let col = match order_col.as_str() {
                "title" => tasks::Column::Title,
                "status" => tasks::Column::Status,
                "priority" => tasks::Column::Priority,
                "complexity" => tasks::Column::Complexity,
                "due_date" => tasks::Column::DueDate,
                "created_at" => tasks::Column::CreatedAt,
                _ => tasks::Column::CreatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };
            query = query.order_by(col, order);
        } else {
            query = query.order_by_desc(tasks::Column::CreatedAt);
        }

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let offset = (payload.page - 1) * payload.limit;

        let models = query
            .left_join(projects::Entity)
            .left_join(milestones::Entity)
            .select_only()
            .column(tasks::Column::Id)
            .column(tasks::Column::ProjectId)
            .column(tasks::Column::MilestoneId)
            .column(tasks::Column::Title)
            .column(tasks::Column::Status)
            .column(tasks::Column::Priority)
            .column(tasks::Column::Complexity)
            .column(tasks::Column::DueDate)
            .column_as(projects::Column::Name, "project_name")
            .column_as(milestones::Column::Name, "milestone_name")
            .column(tasks::Column::CreatedAt)
            .column(tasks::Column::UpdatedAt)
            .limit(payload.limit)
            .offset(offset)
            .into_model::<TaskSummaryDto>()
            .all(self.db.as_ref())
            .await?;

        Ok((models, total_items))
    }

    pub async fn view(&self, user_id: Uuid) -> Result<TaskViewDto, AppError> {
        let (items, total) = self
            .search(
                user_id,
                QueryPayload {
                    page: 1,
                    limit: 10,
                    order_by: Some("created_at".to_string()),
                    order_asc: false,
                    filters: vec![],
                },
            )
            .await?;

        let projects = Projects::find()
            .filter(projects::Column::UserId.eq(user_id))
            .order_by_desc(projects::Column::CreatedAt)
            .limit(10)
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|project| LookupOptionDto {
                id: project.id,
                name: project.name,
            })
            .collect::<Vec<LookupOptionDto>>();

        let project_ids = projects.iter().map(|p| p.id).collect::<Vec<Uuid>>();

        let milestones = if project_ids.is_empty() {
            Vec::new()
        } else {
            Milestones::find()
                .filter(milestones::Column::ProjectId.is_in(project_ids))
                .order_by_desc(milestones::Column::CreatedAt)
                .limit(10)
                .all(self.db.as_ref())
                .await?
                .into_iter()
                .map(|milestone| MilestoneLookupDto {
                    id: milestone.id,
                    project_id: milestone.project_id,
                    name: milestone.name,
                    due_date: milestone.due_date,
                })
                .collect::<Vec<MilestoneLookupDto>>()
        };

        let tags = Tags::find()
            .filter(tags::Column::UserId.eq(user_id))
            .order_by_asc(tags::Column::Name)
            .limit(100)
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|tag| LookupOptionDto {
                id: tag.id,
                name: tag.name,
            })
            .collect::<Vec<LookupOptionDto>>();

        Ok(TaskViewDto {
            items,
            total,
            projects,
            milestones,
            tags,
        })
    }
}

impl FromRequestParts<AppState> for TaskService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(TaskService::new(state.db.clone()))
    }
}
