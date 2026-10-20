use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbBackend, EntityTrait, NotSet, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, Statement, sea_query::Expr,
};
use std::collections::HashSet;
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{
    dtos::{
        attachment::StorageBackedDbWrite,
        hex_color::HexColor,
        project::ProjectStatus,
        query::QueryPayload,
        task::{
            CreateTaskDto, MilestoneLookupDto, TaskDetailDto, TaskProjectDto, TaskSearchField, TaskStatus,
            TaskSummaryDto, TaskTagDto, TaskViewDto, UpdateTaskDto, UpdateTaskStatusDto,
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
    services::attachments::AttachmentService,
    utils::{
        multipart::{MarkdownUploadedFile, replace_markdown_attachment_urls},
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub struct TaskService;

impl TaskService {
    /// Estado pedido: un `status_id` del usuario (con su categoría) o solo la categoría.
    /// Devuelve el id (si se eligió uno concreto) y la categoría resultante.
    async fn resolve_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        status_id: Option<Uuid>,
        status: Option<TaskStatus>,
    ) -> Result<(Option<Uuid>, String), AppError> {
        if let Some(id) = status_id {
            let row = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT category FROM task_statuses WHERE id = $1 AND user_id = $2",
                    [id.into(), user_id.into()],
                ))
                .await?
                .ok_or_else(|| AppError::BadRequest(codes::TASK_STATUS_NOT_FOUND.into()))?;
            return Ok((Some(id), row.try_get("", "category")?));
        }
        Ok((None, status.unwrap_or(TaskStatus::Todo).to_string()))
    }

    /// Nombre, color y clave del estado de una tarea.
    async fn status_presentation(
        db: &impl ConnectionTrait,
        status_id: Uuid,
    ) -> Result<(Option<String>, Option<String>, Option<String>), AppError> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT name, color, key FROM task_statuses WHERE id = $1",
                [status_id.into()],
            ))
            .await?;
        Ok(match row {
            Some(row) => (
                row.try_get("", "name")?,
                row.try_get("", "color")?,
                row.try_get("", "key")?,
            ),
            None => (None, None, None),
        })
    }

    async fn get_task_model(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TaskModel, AppError> {
        let task = Tasks::find()
            .filter(tasks::Column::Id.eq(id))
            .filter(tasks::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::TASK_NOT_FOUND.into()))?;

        Ok(task)
    }

    async fn get_tag_ids_for_task(db: &impl ConnectionTrait, task_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let tag_ids: Vec<Uuid> = TaskTags::find()
            .filter(task_tags::Column::TaskId.eq(task_id))
            .select_only()
            .column(task_tags::Column::TagId)
            .into_tuple()
            .all(db)
            .await?;

        Ok(tag_ids)
    }

    fn parse_tag_color_or_internal(color: &str, tag_id: Uuid) -> Result<HexColor, AppError> {
        HexColor::new(color).map_err(|_| AppError::Internal(format!("tag {tag_id} has an invalid stored color")))
    }

    async fn to_task_detail(db: &impl ConnectionTrait, task: TaskModel) -> Result<TaskDetailDto, AppError> {
        let tag_ids = Self::get_tag_ids_for_task(db, task.id).await?;

        let tags = if tag_ids.is_empty() {
            Vec::new()
        } else {
            Tags::find()
                .filter(tags::Column::UserId.eq(task.user_id))
                .filter(tags::Column::Id.is_in(tag_ids.clone()))
                .order_by_asc(tags::Column::Name)
                .all(db)
                .await?
                .into_iter()
                .map(|tag| {
                    let color = Self::parse_tag_color_or_internal(&tag.color, tag.id)?;
                    Ok(TaskTagDto {
                        id: tag.id,
                        name: tag.name,
                        color,
                    })
                })
                .collect::<Result<Vec<TaskTagDto>, AppError>>()?
        };

        let project_name = if let Some(project_id) = task.project_id {
            Projects::find()
                .filter(projects::Column::Id.eq(project_id))
                .filter(projects::Column::UserId.eq(task.user_id))
                .one(db)
                .await?
                .map(|project| project.name)
        } else {
            None
        };

        let milestone_name = if let Some(milestone_id) = task.milestone_id {
            Milestones::find()
                .filter(milestones::Column::Id.eq(milestone_id))
                .one(db)
                .await?
                .map(|milestone| milestone.name)
        } else {
            None
        };

        let (parent_title, parent_record_number) = if let Some(parent_id) = task.parent_id {
            Tasks::find()
                .filter(tasks::Column::Id.eq(parent_id))
                .filter(tasks::Column::UserId.eq(task.user_id))
                .one(db)
                .await?
                .map(|parent| (Some(parent.title), Some(parent.record_number)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        let (status_name, status_color, status_key) = Self::status_presentation(db, task.status_id).await?;

        Ok(TaskDetailDto {
            id: task.id,
            record_number: task.record_number,
            user_id: task.user_id,
            project_id: task.project_id,
            parent_id: task.parent_id,
            milestone_id: task.milestone_id,
            title: task.title,
            description: task.description,
            status: task.status.try_into().unwrap_or(TaskStatus::Todo),
            status_id: task.status_id,
            status_name,
            status_color,
            status_key,
            priority: task.priority,
            complexity: task.complexity,
            block_parent: task.block_parent,
            due_date: task.due_date,
            project_name,
            milestone_name,
            parent_title,
            parent_record_number,
            created_at: task.created_at,
            updated_at: task.updated_at,
            tag_ids,
            tags,
        })
    }

    async fn ensure_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(pid) = project_id {
            let project = Projects::find()
                .filter(projects::Column::Id.eq(pid))
                .filter(projects::Column::UserId.eq(user_id))
                .one(db)
                .await?;

            project.ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_PROJECT_FORBIDDEN.into()))?;
        }

        Ok(())
    }

    async fn ensure_project_accepts_new_tasks(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(pid) = project_id {
            let project = Projects::find()
                .filter(projects::Column::Id.eq(pid))
                .filter(projects::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_PROJECT_FORBIDDEN.into()))?;

            if project.status == ProjectStatus::Closed.to_string() {
                return Err(AppError::Conflict(codes::PROJECT_CLOSED_NO_TASKS.into()));
            }
        }

        Ok(())
    }

    async fn ensure_parent_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        parent_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(pid) = parent_id {
            let parent = Tasks::find()
                .filter(tasks::Column::Id.eq(pid))
                .filter(tasks::Column::UserId.eq(user_id))
                .one(db)
                .await?;

            let Some(parent) = parent else {
                return Err(AppError::BadRequest(codes::TASK_PARENT_FORBIDDEN.into()));
            };

            if parent.project_id != project_id {
                return Err(AppError::BadRequest(codes::TASK_PARENT_NOT_IN_PROJECT.into()));
            }
        }

        Ok(())
    }

    async fn ensure_milestone_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        milestone_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(mid) = milestone_id {
            let milestone_join = Milestones::find()
                .filter(milestones::Column::Id.eq(mid))
                .find_also_related(Projects)
                .one(db)
                .await?;

            match milestone_join {
                Some((milestone, Some(project))) => {
                    if project.user_id != user_id {
                        return Err(AppError::BadRequest(codes::ATTACHMENT_MILESTONE_FORBIDDEN.into()));
                    }

                    if let Some(pid) = project_id {
                        if milestone.project_id != pid {
                            return Err(AppError::BadRequest(codes::TASK_MILESTONE_NOT_IN_PROJECT.into()));
                        }
                    }
                }
                _ => {
                    return Err(AppError::BadRequest(codes::TASK_MILESTONE_INVALID.into()));
                }
            }
        }

        Ok(())
    }

    async fn ensure_milestone_project_accepts_new_tasks(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        milestone_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(mid) = milestone_id {
            let milestone_join = Milestones::find()
                .filter(milestones::Column::Id.eq(mid))
                .find_also_related(Projects)
                .one(db)
                .await?;

            match milestone_join {
                Some((_milestone, Some(project))) if project.user_id == user_id => {
                    if project.status == ProjectStatus::Closed.to_string() {
                        return Err(AppError::Conflict(codes::PROJECT_CLOSED_NO_TASKS.into()));
                    }
                }
                _ => return Err(AppError::BadRequest(codes::TASK_MILESTONE_INVALID.into())),
            }
        }

        Ok(())
    }

    async fn validate_and_dedup_tags(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        tag_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, AppError> {
        if tag_ids.is_empty() {
            return Ok(Vec::new());
        }

        let deduped_ids: Vec<Uuid> = tag_ids.iter().copied().collect::<HashSet<Uuid>>().into_iter().collect();

        let found_count = Tags::find()
            .filter(tags::Column::UserId.eq(user_id))
            .filter(tags::Column::Id.is_in(deduped_ids.clone()))
            .count(db)
            .await?;

        if found_count != deduped_ids.len() as u64 {
            return Err(AppError::BadRequest(codes::TASK_TAGS_INVALID.into()));
        }

        Ok(deduped_ids)
    }

    async fn create_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateTaskDto,
    ) -> Result<TaskModel, AppError> {
        Self::ensure_project_ownership(db, user_id, dto.project_id).await?;
        Self::ensure_parent_ownership(db, user_id, dto.parent_id, dto.project_id).await?;
        Self::ensure_milestone_ownership(db, user_id, dto.milestone_id, dto.project_id).await?;
        Self::ensure_project_accepts_new_tasks(db, user_id, dto.project_id).await?;
        Self::ensure_milestone_project_accepts_new_tasks(db, user_id, dto.milestone_id).await?;
        let safe_tag_ids = Self::validate_and_dedup_tags(db, user_id, &dto.tag_ids).await?;
        let (status_id, status) = Self::resolve_status(db, user_id, dto.status_id, dto.status).await?;

        let task = tasks::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            parent_id: Set(dto.parent_id),
            milestone_id: Set(dto.milestone_id),
            title: Set(dto.title),
            description: Set(dto.description),
            // La base sincroniza status (categoría) y status_id: con status_id manda el estado.
            status: Set(status),
            status_id: status_id.map(Set).unwrap_or(NotSet),
            priority: Set(dto.priority),
            complexity: Set(dto.complexity),
            block_parent: Set(dto.block_parent),
            due_date: Set(dto.due_date),
            ..Default::default()
        };

        let created_task = task.insert(db).await.map_err(AppError::from)?;

        if !safe_tag_ids.is_empty() {
            let links: Vec<task_tags::ActiveModel> = safe_tag_ids
                .iter()
                .copied()
                .map(|tag_id| task_tags::ActiveModel {
                    task_id: Set(created_task.id),
                    tag_id: Set(tag_id),
                })
                .collect();

            TaskTags::insert_many(links).exec(db).await.map_err(AppError::from)?;
        }

        Ok(created_task)
    }

    pub async fn create_in_tx(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateTaskDto,
    ) -> Result<TaskDetailDto, AppError> {
        let created_task = Self::create_with_connection(db, user_id, dto).await?;
        Self::get_by_id(db, user_id, created_task.id).await
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateTaskDto,
    ) -> Result<TaskDetailDto, AppError> {
        let created_task = Self::create_with_connection(db, user_id, dto).await?;
        Self::get_by_id(db, user_id, created_task.id).await
    }

    pub async fn create_with_markdown_files(
        db: &impl ConnectionTrait,
        storage: &dyn object_store::ObjectStore,
        user_id: Uuid,
        dto: CreateTaskDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<StorageBackedDbWrite<TaskDetailDto>, AppError> {
        if files.is_empty() {
            return Ok(StorageBackedDbWrite::without_storage(
                Self::create(db, user_id, dto).await?,
            ));
        }

        let created_task = Self::create_with_connection(db, user_id, dto).await?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            db,
            storage,
            user_id,
            created_task.project_id,
            Some(created_task.id),
            None,
            None,
            files,
        )
        .await?;

        let next_description = replace_markdown_attachment_urls(created_task.description.clone(), &upload.replacements);
        let mut active_task: tasks::ActiveModel = created_task.into();
        active_task.description = Set(next_description);
        let updated_task = match active_task.update(db).await.map_err(AppError::from) {
            Ok(updated_task) => updated_task,
            Err(error) => {
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        Ok(StorageBackedDbWrite {
            value: Self::get_by_id(db, user_id, updated_task.id).await?,
            storage_keys: upload.storage_keys,
        })
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TaskDetailDto, AppError> {
        let task = Self::get_task_model(db, user_id, id).await?;
        Self::to_task_detail(db, task).await
    }

    async fn update_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateTaskDto,
    ) -> Result<TaskModel, AppError> {
        if dto.parent_id == Some(id) {
            return Err(AppError::BadRequest(codes::TASK_SELF_PARENT.into()));
        }

        let task = Self::get_task_model(db, user_id, id).await?;
        Self::ensure_project_ownership(db, user_id, dto.project_id).await?;
        Self::ensure_parent_ownership(db, user_id, dto.parent_id, dto.project_id).await?;
        Self::ensure_milestone_ownership(db, user_id, dto.milestone_id, dto.project_id).await?;
        let safe_tag_ids = Self::validate_and_dedup_tags(db, user_id, &dto.tag_ids).await?;

        let (status_id, status) = Self::resolve_status(db, user_id, dto.status_id, dto.status).await?;
        if status == TaskStatus::Done.to_string() && task.status != TaskStatus::Done.to_string() {
            Self::ensure_no_blocking_children(db, id).await?;
        }

        let mut active_task: tasks::ActiveModel = task.into();
        active_task.project_id = Set(dto.project_id);
        active_task.parent_id = Set(dto.parent_id);
        active_task.milestone_id = Set(dto.milestone_id);
        active_task.title = Set(dto.title);
        active_task.description = Set(dto.description);
        match status_id {
            Some(status_id) => active_task.status_id = Set(status_id),
            None => active_task.status = Set(status),
        }
        active_task.priority = Set(dto.priority);
        active_task.complexity = Set(dto.complexity);
        active_task.block_parent = Set(dto.block_parent);
        active_task.due_date = Set(dto.due_date);

        let updated_task = active_task.update(db).await.map_err(AppError::from)?;

        TaskTags::delete_many()
            .filter(task_tags::Column::TaskId.eq(updated_task.id))
            .exec(db)
            .await
            .map_err(AppError::from)?;

        if !safe_tag_ids.is_empty() {
            let links: Vec<task_tags::ActiveModel> = safe_tag_ids
                .iter()
                .copied()
                .map(|tag_id| task_tags::ActiveModel {
                    task_id: Set(updated_task.id),
                    tag_id: Set(tag_id),
                })
                .collect();

            TaskTags::insert_many(links).exec(db).await.map_err(AppError::from)?;
        }

        Ok(updated_task)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateTaskDto,
    ) -> Result<TaskDetailDto, AppError> {
        let updated_task = Self::update_with_connection(db, user_id, id, dto).await?;
        Self::get_by_id(db, user_id, updated_task.id).await
    }

    pub async fn update_with_markdown_files(
        db: &impl ConnectionTrait,
        storage: &dyn object_store::ObjectStore,
        user_id: Uuid,
        id: Uuid,
        mut dto: UpdateTaskDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<StorageBackedDbWrite<TaskDetailDto>, AppError> {
        if files.is_empty() {
            return Ok(StorageBackedDbWrite::without_storage(
                Self::update(db, user_id, id, dto).await?,
            ));
        }

        let task = Self::get_task_model(db, user_id, id).await?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            db,
            storage,
            user_id,
            task.project_id,
            Some(task.id),
            None,
            None,
            files,
        )
        .await?;

        dto.description = replace_markdown_attachment_urls(dto.description, &upload.replacements);
        let updated_task = match Self::update_with_connection(db, user_id, id, dto).await {
            Ok(updated_task) => updated_task,
            Err(error) => {
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        Ok(StorageBackedDbWrite {
            value: Self::get_by_id(db, user_id, updated_task.id).await?,
            storage_keys: upload.storage_keys,
        })
    }

    pub async fn change_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateTaskStatusDto,
    ) -> Result<(), AppError> {
        let task = Self::get_task_model(db, user_id, id).await?;
        let (status_id, status) = Self::resolve_status(db, user_id, dto.status_id, dto.status).await?;

        if status == TaskStatus::Done.to_string() {
            Self::ensure_no_blocking_children(db, id).await?;
        }

        let mut active_task: tasks::ActiveModel = task.into();
        match status_id {
            Some(status_id) => active_task.status_id = Set(status_id),
            None => active_task.status = Set(status),
        }
        let _ = active_task.update(db).await.map_err(AppError::from)?;

        Ok(())
    }

    async fn ensure_no_blocking_children(db: &impl ConnectionTrait, parent_id: Uuid) -> Result<(), AppError> {
        let blocking_count = Tasks::find()
            .filter(tasks::Column::ParentId.eq(parent_id))
            .filter(tasks::Column::BlockParent.eq(true))
            .filter(tasks::Column::Status.is_not_in([TaskStatus::Done.to_string(), TaskStatus::Cancelled.to_string()]))
            .count(db)
            .await
            .map_err(AppError::from)?;

        if blocking_count > 0 {
            return Err(AppError::BadRequest(codes::TASK_BLOCKING_SUBTASKS.into()));
        }

        Ok(())
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let task = Self::get_task_model(db, user_id, id).await?;

        tasks::ActiveModel {
            id: Set(task.id),
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
    ) -> Result<(Vec<TaskSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(tasks::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<TaskSearchField>()
                .ok()
                .map(TaskSearchField::filter_field)
        })?;

        let query = Tasks::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            TaskSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<TaskSearchField>()
                    .ok()
                    .map(TaskSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let models = apply_pagination(
            query
                .left_join(projects::Entity)
                .left_join(milestones::Entity)
                .select_only()
                .column(tasks::Column::Id)
                .column(tasks::Column::RecordNumber)
                .column(tasks::Column::ProjectId)
                .column(tasks::Column::MilestoneId)
                .column(tasks::Column::Title)
                .column(tasks::Column::Status)
                .column(tasks::Column::StatusId)
                .column_as(
                    Expr::cust("(SELECT s.name FROM task_statuses s WHERE s.id = tasks.status_id)"),
                    "status_name",
                )
                .column_as(
                    Expr::cust("(SELECT s.color FROM task_statuses s WHERE s.id = tasks.status_id)"),
                    "status_color",
                )
                .column_as(
                    Expr::cust("(SELECT s.key FROM task_statuses s WHERE s.id = tasks.status_id)"),
                    "status_key",
                )
                .column(tasks::Column::Priority)
                .column(tasks::Column::Complexity)
                .column(tasks::Column::DueDate)
                .column_as(projects::Column::Name, "project_name")
                .column_as(milestones::Column::Name, "milestone_name")
                .column(tasks::Column::CreatedAt)
                .column(tasks::Column::UpdatedAt),
            &payload,
        )
        .into_model::<TaskSummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    pub async fn view(db: &impl ConnectionTrait, user_id: Uuid) -> Result<TaskViewDto, AppError> {
        let (items, total) = Self::search(
            db,
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
            .all(db)
            .await?
            .into_iter()
            .map(|project| TaskProjectDto {
                id: project.id,
                name: project.name,
            })
            .collect::<Vec<TaskProjectDto>>();

        let project_ids = projects.iter().map(|p| p.id).collect::<Vec<Uuid>>();

        let milestones = if project_ids.is_empty() {
            Vec::new()
        } else {
            Milestones::find()
                .filter(milestones::Column::ProjectId.is_in(project_ids))
                .order_by_desc(milestones::Column::CreatedAt)
                .limit(10)
                .all(db)
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
            .all(db)
            .await?
            .into_iter()
            .map(|tag| {
                let color = Self::parse_tag_color_or_internal(&tag.color, tag.id)?;
                Ok(TaskTagDto {
                    id: tag.id,
                    name: tag.name,
                    color,
                })
            })
            .collect::<Result<Vec<TaskTagDto>, AppError>>()?;

        Ok(TaskViewDto {
            items,
            total,
            projects,
            milestones,
            tags,
        })
    }
}
