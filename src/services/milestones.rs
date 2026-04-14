use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        milestone::{MilestoneDto, MilestoneSearchField, ProjectMilestoneSummaryDto},
        query::QueryPayload,
    },
    entities::{
        milestones::{self, Model as MilestoneModel},
        prelude::{Milestones, Projects},
        projects,
    },
    error::AppError,
    state::AppState,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

#[derive(Clone)]
pub struct MilestoneService {
    pub db: Arc<DatabaseConnection>,
}

const PROJECT_MILESTONES_PREVIEW_LIMIT: u64 = 10;

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

    async fn check_client_project_ownership(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
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

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let models = apply_pagination(query, &payload).all(self.db.as_ref()).await?;

        Ok((models, total_items))
    }

    pub async fn search_by_project(
        &self,
        user_id: Uuid,
        project_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<MilestoneModel>, u64), AppError> {
        self.check_project_ownership(user_id, project_id).await?;

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

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let models = apply_pagination(query, &payload).all(self.db.as_ref()).await?;

        Ok((models, total_items))
    }

    pub async fn client_search_project_milestones(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ProjectMilestoneSummaryDto>, u64), AppError> {
        self.check_client_project_ownership(user_id, client_id, project_id)
            .await?;

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

        let total_items = query.clone().count(self.db.as_ref()).await?;

        let models = apply_pagination(
            query
                .select_only()
                .column(milestones::Column::Id)
                .column(milestones::Column::Name)
                .column(milestones::Column::DueDate)
                .column(milestones::Column::CreatedAt)
                .column(milestones::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ProjectMilestoneSummaryDto>()
        .all(self.db.as_ref())
        .await?;

        Ok((models, total_items))
    }

    pub async fn list_project_milestones_preview(
        &self,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        self.check_project_ownership(user_id, project_id).await?;
        self.fetch_project_milestones_preview(project_id).await
    }

    pub async fn list_client_project_milestones_preview(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        self.check_client_project_ownership(user_id, client_id, project_id)
            .await?;
        self.fetch_project_milestones_preview(project_id).await
    }

    async fn fetch_project_milestones_preview(
        &self,
        project_id: Uuid,
    ) -> Result<Vec<ProjectMilestoneSummaryDto>, AppError> {
        let milestones = Milestones::find()
            .filter(milestones::Column::ProjectId.eq(project_id))
            .select_only()
            .column(milestones::Column::Id)
            .column(milestones::Column::Name)
            .column(milestones::Column::DueDate)
            .column(milestones::Column::CreatedAt)
            .column(milestones::Column::UpdatedAt)
            .order_by_asc(milestones::Column::DueDate)
            .order_by_asc(milestones::Column::CreatedAt)
            .limit(PROJECT_MILESTONES_PREVIEW_LIMIT)
            .into_model::<ProjectMilestoneSummaryDto>()
            .all(self.db.as_ref())
            .await?;

        Ok(milestones)
    }
}

impl FromRequestParts<AppState> for MilestoneService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(MilestoneService::new(state.db.clone()))
    }
}
