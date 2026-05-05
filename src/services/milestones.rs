use object_store::ObjectStore;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    dtos::{
        milestone::{
            CreateMilestoneDto, MilestoneSearchField, MilestoneStatus, ProjectMilestoneSummaryDto, UpdateMilestoneDto,
        },
        project::ProjectStatus,
        query::QueryPayload,
    },
    entities::{
        milestones::{self, Model as MilestoneModel},
        prelude::{Milestones, Projects},
        projects,
    },
    error::AppError,
    services::attachments::AttachmentService,
    utils::{
        multipart::{MarkdownUploadedFile, replace_markdown_attachment_urls},
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub struct MilestoneService;

const PROJECT_MILESTONES_PREVIEW_LIMIT: u64 = 10;

impl MilestoneService {
    async fn check_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        project.ok_or_else(|| AppError::NotFound("project not found or no permission".to_string()))?;

        Ok(())
    }

    async fn check_project_accepts_new_milestones(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("project not found or no permission".to_string()))?;

        if project.status == ProjectStatus::Closed.to_string() {
            return Err(AppError::Conflict(
                "closed projects cannot accept new milestones".to_string(),
            ));
        }

        Ok(())
    }

    async fn check_client_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
            .one(db)
            .await?;

        if project.is_none() {
            return Err(AppError::NotFound("project not found or no permission".to_string()));
        }

        Ok(())
    }

    async fn check_milestone_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        milestone_id: Uuid,
    ) -> Result<MilestoneModel, AppError> {
        // Find milestone and join project to verify user
        let milestone_opt = Milestones::find()
            .filter(milestones::Column::Id.eq(milestone_id))
            .find_also_related(Projects)
            .one(db)
            .await?;

        match milestone_opt {
            Some((milestone, Some(project))) => {
                if project.user_id != user_id {
                    return Err(AppError::NotFound("milestone not found or no permission".to_string()));
                }
                Ok(milestone)
            }
            _ => Err(AppError::NotFound("milestone not found".to_string())),
        }
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateMilestoneDto,
    ) -> Result<MilestoneModel, AppError> {
        Self::check_project_accepts_new_milestones(db, user_id, dto.project_id).await?;

        let status = dto.status.unwrap_or(MilestoneStatus::Open).to_string();

        let milestone = milestones::ActiveModel {
            project_id: Set(dto.project_id),
            name: Set(dto.name),
            description: Set(dto.description),
            status: Set(status),
            due_date: Set(dto.due_date),
            ..Default::default()
        };

        milestone.insert(db).await.map_err(AppError::Database)
    }

    pub async fn create_with_markdown_files(
        db: &(impl ConnectionTrait + TransactionTrait),
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateMilestoneDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<MilestoneModel, AppError> {
        if files.is_empty() {
            return Self::create(db, user_id, dto).await;
        }

        let txn = db.begin().await.map_err(AppError::Database)?;
        let milestone = Self::create(&txn, user_id, dto).await?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            &txn,
            storage,
            user_id,
            Some(milestone.project_id),
            None,
            Some(milestone.id),
            None,
            files,
        )
        .await?;

        let next_description = replace_markdown_attachment_urls(milestone.description.clone(), &upload.replacements);
        let mut active_milestone: milestones::ActiveModel = milestone.into();
        active_milestone.description = Set(next_description);
        let updated = match active_milestone.update(&txn).await.map_err(AppError::Database) {
            Ok(updated) => updated,
            Err(error) => {
                if let Err(rollback_error) = txn.rollback().await {
                    tracing::warn!(error = %rollback_error, "Milestone markdown create rollback failed");
                }
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        if let Err(error) = txn.commit().await {
            AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
            return Err(AppError::Database(error));
        }
        Ok(updated)
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<MilestoneModel, AppError> {
        Self::check_milestone_ownership(db, user_id, id).await
    }

    async fn update_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateMilestoneDto,
    ) -> Result<MilestoneModel, AppError> {
        let milestone = Self::check_milestone_ownership(db, user_id, id).await?;

        if milestone.project_id != dto.project_id {
            Self::check_project_accepts_new_milestones(db, user_id, dto.project_id).await?;
        }

        let mut active_milestone: milestones::ActiveModel = milestone.into();
        active_milestone.project_id = Set(dto.project_id);
        active_milestone.name = Set(dto.name);
        active_milestone.description = Set(dto.description);
        active_milestone.status = Set(dto.status.to_string());
        active_milestone.due_date = Set(dto.due_date);

        active_milestone.update(db).await.map_err(AppError::Database)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateMilestoneDto,
    ) -> Result<MilestoneModel, AppError> {
        Self::update_with_connection(db, user_id, id, dto).await
    }

    pub async fn update_with_markdown_files(
        db: &(impl ConnectionTrait + TransactionTrait),
        storage: &dyn ObjectStore,
        user_id: Uuid,
        id: Uuid,
        mut dto: UpdateMilestoneDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<MilestoneModel, AppError> {
        if files.is_empty() {
            return Self::update(db, user_id, id, dto).await;
        }

        let txn = db.begin().await.map_err(AppError::Database)?;
        let milestone = Self::check_milestone_ownership(&txn, user_id, id).await?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            &txn,
            storage,
            user_id,
            Some(milestone.project_id),
            None,
            Some(milestone.id),
            None,
            files,
        )
        .await?;

        dto.description = replace_markdown_attachment_urls(dto.description, &upload.replacements);
        let updated_milestone = match Self::update_with_connection(&txn, user_id, id, dto).await {
            Ok(updated_milestone) => updated_milestone,
            Err(error) => {
                if let Err(rollback_error) = txn.rollback().await {
                    tracing::warn!(error = %rollback_error, "Milestone markdown update rollback failed");
                }
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        if let Err(error) = txn.commit().await {
            AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
            return Err(AppError::Database(error));
        }

        Ok(updated_milestone)
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let milestone = Self::check_milestone_ownership(db, user_id, id).await?;

        milestones::ActiveModel {
            id: Set(milestone.id),
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
    ) -> Result<(Vec<MilestoneModel>, u64), AppError> {
        // We only return milestones for projects that belong to the user
        let project_ids: Vec<Uuid> = Projects::find()
            .filter(projects::Column::UserId.eq(user_id))
            .select_only()
            .column(projects::Column::Id)
            .into_tuple()
            .all(db)
            .await?;

        let base_condition = Condition::all().add(milestones::Column::ProjectId.is_in(project_ids));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<MilestoneSearchField>()
                .ok()
                .map(MilestoneSearchField::filter_field)
        })?;

        let query = Milestones::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            MilestoneSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<MilestoneSearchField>()
                    .ok()
                    .map(MilestoneSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(query, &payload).all(db).await?;

        Ok((models, total_items))
    }

    pub async fn search_by_project(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<MilestoneModel>, u64), AppError> {
        Self::check_project_ownership(db, user_id, project_id).await?;

        let base_condition = Condition::all().add(milestones::Column::ProjectId.eq(project_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<MilestoneSearchField>()
                .ok()
                .map(MilestoneSearchField::filter_field)
        })?;

        let query = Milestones::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            MilestoneSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<MilestoneSearchField>()
                    .ok()
                    .map(MilestoneSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(query, &payload).all(db).await?;

        Ok((models, total_items))
    }

    pub async fn client_search_project_milestones(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ProjectMilestoneSummaryDto>, u64), AppError> {
        Self::check_client_project_ownership(db, user_id, client_id, project_id).await?;

        let base_condition = Condition::all().add(milestones::Column::ProjectId.eq(project_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<MilestoneSearchField>()
                .ok()
                .map(MilestoneSearchField::filter_field)
        })?;

        let query = Milestones::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            MilestoneSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<MilestoneSearchField>()
                    .ok()
                    .map(MilestoneSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let models = apply_pagination(
            query
                .select_only()
                .column(milestones::Column::Id)
                .column(milestones::Column::Name)
                .column(milestones::Column::Description)
                .column(milestones::Column::Status)
                .column(milestones::Column::DueDate)
                .column(milestones::Column::CreatedAt)
                .column(milestones::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ProjectMilestoneSummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    pub async fn list_project_milestones_preview(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        Self::check_project_ownership(db, user_id, project_id).await?;
        Self::fetch_project_milestones_preview(db, project_id).await
    }

    pub async fn list_client_project_milestones_preview(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        Self::check_client_project_ownership(db, user_id, client_id, project_id).await?;
        Self::fetch_project_milestones_preview(db, project_id).await
    }

    async fn fetch_project_milestones_preview(
        db: &impl ConnectionTrait,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        let milestones = Milestones::find()
            .filter(milestones::Column::ProjectId.eq(project_id))
            .select_only()
            .column(milestones::Column::Id)
            .column(milestones::Column::Name)
            .column(milestones::Column::Description)
            .column(milestones::Column::Status)
            .column(milestones::Column::DueDate)
            .column(milestones::Column::CreatedAt)
            .column(milestones::Column::UpdatedAt)
            .order_by_asc(milestones::Column::DueDate)
            .order_by_asc(milestones::Column::CreatedAt)
            .limit(PROJECT_MILESTONES_PREVIEW_LIMIT)
            .into_model::<ProjectMilestoneSummaryDto>()
            .all(db)
            .await?;

        Ok(milestones)
    }
}
