use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use uuid::Uuid;

use crate::{
    dtos::{
        change_request::{
            AssessChangeRequestDto, ChangeRequestDetailDto, ChangeRequestDetailRow, ChangeRequestOrigin,
            ChangeRequestSearchField, ChangeRequestStatus, ChangeRequestSummaryDto, ClientApproveChangeRequestDto,
            ClientChangeRequestDetailDto, ClientChangeRequestDetailRow, ClientChangeRequestSearchField,
            ClientChangeRequestSummaryDto, ClientCreateChangeRequestDto, CreateChangeRequestDto,
            UpdateChangeRequestDto,
        },
        query::QueryPayload,
    },
    entities::{
        change_requests::{self, Model as ChangeRequestModel},
        clients,
        prelude::{ChangeRequests, Clients, Projects, Tasks},
        projects, tasks,
    },
    error::AppError,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub struct ChangeRequestService;

impl ChangeRequestService {
    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateChangeRequestDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        Self::ensure_project_client_pair(db, user_id, dto.project_id, dto.client_id).await?;
        Self::ensure_task_ownership(db, user_id, dto.project_id, dto.task_id).await?;

        let change_request = change_requests::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            client_id: Set(dto.client_id),
            task_id: Set(dto.task_id),
            origin: Set(ChangeRequestOrigin::FreelancerDashboard.to_string()),
            title: Set(dto.title),
            client_request_description: Set(dto.client_request_description),
            status: Set(ChangeRequestStatus::Draft.to_string()),
            ..Default::default()
        };

        let created = change_request.insert(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn create_by_client(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        dto: ClientCreateChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        Self::ensure_client_project_ownership(db, user_id, client_id, dto.project_id).await?;

        let change_request = change_requests::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            client_id: Set(client_id),
            task_id: Set(None),
            origin: Set(ChangeRequestOrigin::ClientPortal.to_string()),
            title: Set(dto.title),
            client_request_description: Set(dto.client_request_description),
            status: Set(ChangeRequestStatus::Submitted.to_string()),
            ..Default::default()
        };

        let created = change_request.insert(db).await.map_err(AppError::Database)?;
        Self::get_client_by_id(db, user_id, client_id, created.id).await
    }

    pub async fn get_by_id(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let row: ChangeRequestDetailRow = ChangeRequests::find()
            .filter(change_requests::Column::Id.eq(id))
            .filter(change_requests::Column::UserId.eq(user_id))
            .left_join(projects::Entity)
            .left_join(clients::Entity)
            .select_only()
            .column(change_requests::Column::Id)
            .column(change_requests::Column::RecordNumber)
            .column(change_requests::Column::UserId)
            .column(change_requests::Column::ProjectId)
            .column(change_requests::Column::ClientId)
            .column(change_requests::Column::TaskId)
            .column(change_requests::Column::Title)
            .column(change_requests::Column::ClientRequestDescription)
            .column(change_requests::Column::TechnicalAssessment)
            .column(change_requests::Column::ImpactHours)
            .column(change_requests::Column::ImpactCost)
            .column(change_requests::Column::ScheduleImpactDays)
            .column(change_requests::Column::Origin)
            .column(change_requests::Column::Status)
            .column(change_requests::Column::ClientApprovedAt)
            .column(change_requests::Column::ClientSignatureIp)
            .column(change_requests::Column::CreatedAt)
            .column(change_requests::Column::UpdatedAt)
            .column_as(projects::Column::Name, "project_name")
            .column_as(clients::Column::Name, "client_name")
            .into_model::<ChangeRequestDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Solicitud de cambio no encontrada".to_string()))?;

        Ok(Self::map_detail_row(row))
    }

    pub async fn get_client_by_id(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let row: ClientChangeRequestDetailRow = ChangeRequests::find()
            .filter(change_requests::Column::Id.eq(id))
            .filter(change_requests::Column::UserId.eq(user_id))
            .filter(change_requests::Column::ClientId.eq(client_id))
            .left_join(projects::Entity)
            .select_only()
            .column(change_requests::Column::Id)
            .column(change_requests::Column::RecordNumber)
            .column(change_requests::Column::ProjectId)
            .column(change_requests::Column::TaskId)
            .column(change_requests::Column::Title)
            .column(change_requests::Column::ClientRequestDescription)
            .column(change_requests::Column::TechnicalAssessment)
            .column(change_requests::Column::ImpactHours)
            .column(change_requests::Column::ImpactCost)
            .column(change_requests::Column::ScheduleImpactDays)
            .column(change_requests::Column::Status)
            .column(change_requests::Column::ClientApprovedAt)
            .column(change_requests::Column::ClientSignatureIp)
            .column(change_requests::Column::CreatedAt)
            .column(change_requests::Column::UpdatedAt)
            .column_as(projects::Column::Name, "project_name")
            .into_model::<ClientChangeRequestDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Solicitud de cambio no encontrada".to_string()))?;

        Ok(Self::map_client_detail_row(row))
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChangeRequestDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if model.version != dto.version {
            return Err(AppError::Conflict(
                "El cliente actualizó el registro de cambio".to_string(),
            ));
        }

        if !matches!(
            status,
            ChangeRequestStatus::Draft | ChangeRequestStatus::Submitted | ChangeRequestStatus::Assessed
        ) {
            return Err(AppError::Conflict(
                "No se puede editar una solicitud de cambio finalizada".to_string(),
            ));
        }

        Self::ensure_project_client_pair(db, user_id, dto.project_id, dto.client_id).await?;
        Self::ensure_task_ownership(db, user_id, dto.project_id, dto.task_id).await?;

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.project_id = Set(dto.project_id);
        active_model.client_id = Set(dto.client_id);
        active_model.task_id = Set(dto.task_id);
        active_model.title = Set(dto.title);
        active_model.client_request_description = Set(dto.client_request_description);
        active_model.technical_assessment = Set(dto.technical_assessment);
        active_model.impact_hours = Set(dto.impact_hours);
        active_model.impact_cost = Set(dto.impact_cost);
        active_model.schedule_impact_days = Set(dto.schedule_impact_days);

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn submit(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Draft {
            return Err(AppError::Conflict(
                "Solo una solicitud en borrador puede enviarse".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Submitted.to_string());

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn assess(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: AssessChangeRequestDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Submitted {
            return Err(AppError::Conflict(
                "Solo una solicitud enviada puede evaluarse".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.technical_assessment = Set(Some(dto.technical_assessment));
        active_model.impact_hours = Set(dto.impact_hours);
        active_model.impact_cost = Set(dto.impact_cost);
        active_model.schedule_impact_days = Set(dto.schedule_impact_days);
        active_model.status = Set(ChangeRequestStatus::Assessed.to_string());

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn mark_implemented(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Approved {
            return Err(AppError::Conflict(
                "Solo una solicitud aprobada puede marcarse como implementada".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Implemented.to_string());

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn cancel(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if matches!(
            status,
            ChangeRequestStatus::Implemented | ChangeRequestStatus::Cancelled
        ) {
            return Err(AppError::Conflict(
                "La solicitud ya está finalizada y no puede cancelarse".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Cancelled.to_string());

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn client_approve(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
        dto: ClientApproveChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_client(db, user_id, client_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Assessed {
            return Err(AppError::Conflict(
                "Solo una solicitud evaluada puede aprobarse".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Approved.to_string());
        active_model.client_approved_at = Set(Some(Self::now_with_offset()));
        active_model.client_signature_ip = Set(dto.client_signature_ip);

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn client_decline(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_client(db, user_id, client_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Assessed {
            return Err(AppError::Conflict(
                "Solo una solicitud evaluada puede rechazarse".to_string(),
            ));
        }

        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Declined.to_string());
        active_model.client_approved_at = Set(None);
        active_model.client_signature_ip = Set(None);

        let updated = active_model.update(db).await.map_err(AppError::Database)?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ChangeRequestSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(change_requests::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ChangeRequestSearchField>()
                .ok()
                .map(ChangeRequestSearchField::filter_field)
        })?;

        let query = ChangeRequests::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ChangeRequestSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ChangeRequestSearchField>()
                    .ok()
                    .map(ChangeRequestSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let items = apply_pagination(
            query
                .left_join(projects::Entity)
                .left_join(clients::Entity)
                .select_only()
                .column(change_requests::Column::Id)
                .column(change_requests::Column::RecordNumber)
                .column(change_requests::Column::ProjectId)
                .column(change_requests::Column::ClientId)
                .column(change_requests::Column::TaskId)
                .column(change_requests::Column::Title)
                .column(change_requests::Column::Origin)
                .column(change_requests::Column::Status)
                .column(change_requests::Column::CreatedAt)
                .column(change_requests::Column::UpdatedAt)
                .column_as(projects::Column::Name, "project_name")
                .column_as(clients::Column::Name, "client_name"),
            &payload,
        )
        .into_model::<ChangeRequestSummaryDto>()
        .all(db)
        .await?;

        Ok((items, total_items))
    }

    pub async fn client_search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ClientChangeRequestSummaryDto>, u64), AppError> {
        let base_condition = Condition::all()
            .add(change_requests::Column::UserId.eq(user_id))
            .add(change_requests::Column::ClientId.eq(client_id));

        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ClientChangeRequestSearchField>()
                .ok()
                .map(ClientChangeRequestSearchField::filter_field)
        })?;

        let query = ChangeRequests::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ClientChangeRequestSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ClientChangeRequestSearchField>()
                    .ok()
                    .map(ClientChangeRequestSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let items = apply_pagination(
            query
                .left_join(projects::Entity)
                .select_only()
                .column(change_requests::Column::Id)
                .column(change_requests::Column::RecordNumber)
                .column(change_requests::Column::ProjectId)
                .column(change_requests::Column::TaskId)
                .column(change_requests::Column::Title)
                .column(change_requests::Column::Status)
                .column(change_requests::Column::CreatedAt)
                .column(change_requests::Column::UpdatedAt)
                .column_as(projects::Column::Name, "project_name"),
            &payload,
        )
        .into_model::<ClientChangeRequestSummaryDto>()
        .all(db)
        .await?;

        Ok((items, total_items))
    }

    async fn get_model_by_user(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestModel, AppError> {
        let model = ChangeRequests::find()
            .filter(change_requests::Column::Id.eq(id))
            .filter(change_requests::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Solicitud de cambio no encontrada".to_string()))?;

        Ok(model)
    }

    async fn get_model_by_client(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
    ) -> Result<ChangeRequestModel, AppError> {
        let model = ChangeRequests::find()
            .filter(change_requests::Column::Id.eq(id))
            .filter(change_requests::Column::UserId.eq(user_id))
            .filter(change_requests::Column::ClientId.eq(client_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Solicitud de cambio no encontrada".to_string()))?;

        Ok(model)
    }

    async fn ensure_project_client_pair(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        client_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado o no tienes permiso".to_string()))?;

        let client_exists = Clients::find()
            .filter(clients::Column::Id.eq(client_id))
            .filter(clients::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .is_some();

        if !client_exists {
            return Err(AppError::NotFound(
                "Cliente no encontrado o no tienes permiso".to_string(),
            ));
        }

        if project.client_id != Some(client_id) {
            return Err(AppError::BadRequest(
                "El proyecto no está asignado al cliente indicado".to_string(),
            ));
        }

        Ok(())
    }

    async fn ensure_client_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project_exists = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
            .one(db)
            .await?
            .is_some();

        if !project_exists {
            return Err(AppError::NotFound(
                "Proyecto no encontrado o no tienes permiso".to_string(),
            ));
        }

        Ok(())
    }

    async fn ensure_task_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        task_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(task_id) = task_id {
            let task = Tasks::find()
                .filter(tasks::Column::Id.eq(task_id))
                .filter(tasks::Column::UserId.eq(user_id))
                .one(db)
                .await?
                .ok_or_else(|| AppError::BadRequest("Tarea inválida o sin permisos".to_string()))?;

            if task.project_id != Some(project_id) {
                return Err(AppError::BadRequest(
                    "La tarea no pertenece al proyecto indicado".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn parse_status(raw: &str) -> Result<ChangeRequestStatus, AppError> {
        ChangeRequestStatus::try_from(raw.to_string())
            .map_err(|_| AppError::Internal(format!("Estado inválido almacenado en change_requests: {}", raw)))
    }

    fn now_with_offset() -> DateTime<FixedOffset> {
        Utc::now().fixed_offset()
    }

    fn map_detail_row(row: ChangeRequestDetailRow) -> ChangeRequestDetailDto {
        ChangeRequestDetailDto {
            id: row.id,
            record_number: row.record_number,
            user_id: row.user_id,
            project_id: row.project_id,
            project_name: row.project_name,
            client_id: row.client_id,
            client_name: row.client_name,
            task_id: row.task_id,
            title: row.title,
            client_request_description: row.client_request_description,
            technical_assessment: row.technical_assessment,
            impact_hours: row.impact_hours,
            impact_cost: row.impact_cost,
            schedule_impact_days: row.schedule_impact_days,
            origin: row.origin,
            status: row.status,
            client_approved_at: row.client_approved_at,
            client_signature_ip: row.client_signature_ip,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: row.version,
        }
    }

    fn map_client_detail_row(row: ClientChangeRequestDetailRow) -> ClientChangeRequestDetailDto {
        ClientChangeRequestDetailDto {
            id: row.id,
            record_number: row.record_number,
            project_id: row.project_id,
            project_name: row.project_name,
            task_id: row.task_id,
            title: row.title,
            client_request_description: row.client_request_description,
            technical_assessment: row.technical_assessment,
            impact_hours: row.impact_hours,
            impact_cost: row.impact_cost,
            schedule_impact_days: row.schedule_impact_days,
            status: row.status,
            client_approved_at: row.client_approved_at,
            client_signature_ip: row.client_signature_ip,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
