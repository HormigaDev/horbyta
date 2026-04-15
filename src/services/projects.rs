use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Set, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        project::{
            ClientProjectDetailDto, ClientProjectDetailRow, ClientProjectSearchField, ClientProjectSummaryDto,
            CreateProjectDto, ProjectDetailDto, ProjectDetailRow, ProjectSearchField, ProjectSummaryDto,
            UpdateProjectCategoryDto, UpdateProjectDto, UpdateProjectStatusDto,
        },
        query::QueryPayload,
    },
    entities::{
        categories, clients, milestones,
        prelude::Projects,
        projects::{self, Model as ProjectModel},
    },
    error::AppError,
    services::milestones::MilestoneService,
    state::AppState,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
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
            client_id: Set(dto.client_id),
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

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<ProjectDetailDto, AppError> {
        let row: ProjectDetailRow = Projects::find()
            .filter(projects::Column::Id.eq(id))
            .filter(projects::Column::UserId.eq(user_id))
            .left_join(categories::Entity)
            .left_join(clients::Entity)
            .select_only()
            .column(projects::Column::Id)
            .column(projects::Column::RecordNumber)
            .column(projects::Column::Name)
            .column(projects::Column::CategoryId)
            .column(projects::Column::ClientId)
            .column(projects::Column::Description)
            .column(projects::Column::Status)
            .column(projects::Column::CreatedAt)
            .column(projects::Column::UpdatedAt)
            .column_as(categories::Column::Name, "category_name")
            .column_as(clients::Column::Name, "client_name")
            .into_model::<ProjectDetailRow>()
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado".to_string()))?;

        let milestones = MilestoneService::new(self.db.clone())
            .list_project_milestones_preview(user_id, id)
            .await?;

        let project = ProjectDetailDto {
            id: row.id,
            record_number: row.record_number,
            name: row.name,
            category_id: row.category_id,
            category_name: row.category_name,
            client_id: row.client_id,
            client_name: row.client_name,
            description: row.description,
            status: row.status,
            milestones,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(project)
    }

    pub async fn get_client_project_by_id(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<ClientProjectDetailDto, AppError> {
        let row: ClientProjectDetailRow = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
            .select_only()
            .column(projects::Column::Id)
            .column(projects::Column::RecordNumber)
            .column(projects::Column::Name)
            .column(projects::Column::Description)
            .column(projects::Column::Status)
            .column(projects::Column::CreatedAt)
            .column(projects::Column::UpdatedAt)
            .into_model::<ClientProjectDetailRow>()
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado".to_string()))?;

        let milestones = MilestoneService::new(self.db.clone())
            .list_client_project_milestones_preview(user_id, client_id, project_id)
            .await?;

        Ok(ClientProjectDetailDto {
            id: row.id,
            record_number: row.record_number,
            name: row.name,
            description: row.description,
            status: row.status,
            milestones,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: UpdateProjectDto) -> Result<ProjectModel, AppError> {
        let project = self.find_by_id(user_id, id).await?;
        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let mut active_project: projects::ActiveModel = project.into();
        active_project.name = Set(dto.name);
        active_project.description = Set(dto.description);
        active_project.category_id = Set(dto.category_id);
        active_project.client_id = Set(dto.client_id);
        active_project.status = Set(dto.status.to_string());

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
        let project = self.find_by_id(user_id, id).await?;
        let mut active_project: projects::ActiveModel = project.into();

        active_project.category_id = Set(dto.category_id);

        active_project
            .update(self.db.as_ref())
            .await
            .map_err(AppError::Database)
    }

    pub async fn update_status(
        &self,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectStatusDto,
    ) -> Result<ProjectModel, AppError> {
        let project = self.find_by_id(user_id, id).await?;
        let mut active_project: projects::ActiveModel = project.into();

        active_project.status = Set(dto.status.to_string());

        active_project
            .update(self.db.as_ref())
            .await
            .map_err(AppError::Database)
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let project = self.find_by_id(user_id, id).await?;

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
    ) -> Result<(Vec<ProjectSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(projects::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ProjectSearchField>()
                .ok()
                .map(ProjectSearchField::filter_field)
        })?;

        let query = Projects::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ProjectSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ProjectSearchField>()
                    .ok()
                    .map(ProjectSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(self.db.as_ref()).await?;

        let models = apply_pagination(
            query
                .left_join(categories::Entity)
                .left_join(clients::Entity)
                .select_only()
                .column(projects::Column::Id)
                .column(projects::Column::RecordNumber)
                .column(projects::Column::Name)
                .column(projects::Column::Status)
                .column_as(categories::Column::Id, "category_id")
                .column_as(categories::Column::Name, "category_name")
                .column_as(clients::Column::Id, "client_id")
                .column_as(clients::Column::Name, "client_name")
                .column(projects::Column::CreatedAt)
                .column(projects::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ProjectSummaryDto>()
        .all(self.db.as_ref())
        .await?;

        Ok((models, total_items))
    }

    pub async fn client_search_projects(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ClientProjectSummaryDto>, u64), AppError> {
        let base_condition = Condition::all()
            .add(projects::Column::UserId.eq(user_id))
            .add(projects::Column::ClientId.eq(client_id));

        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ClientProjectSearchField>()
                .ok()
                .map(ClientProjectSearchField::filter_field)
        })?;

        let query = Projects::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ClientProjectSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ClientProjectSearchField>()
                    .ok()
                    .map(ClientProjectSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(self.db.as_ref()).await?;

        let models = apply_pagination(
            query
                .select_only()
                .column(projects::Column::Id)
                .column(projects::Column::RecordNumber)
                .column(projects::Column::Name)
                .column(projects::Column::Status)
                .column(projects::Column::CreatedAt)
                .column(projects::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ClientProjectSummaryDto>()
        .all(self.db.as_ref())
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(&self, user_id: Uuid, id: Uuid) -> Result<ProjectModel, AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado".to_string()))?;

        Ok(project)
    }
}

impl FromRequestParts<AppState> for ProjectService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(ProjectService::new(state.db.clone()))
    }
}
