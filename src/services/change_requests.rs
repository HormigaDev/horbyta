use std::str::FromStr;

use chrono::{DateTime, FixedOffset, Utc};
use object_store::ObjectStore;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbBackend, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, Set, Statement,
};
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{
    dtos::scope::ClassifyTaskDto,
    dtos::{
        attachment::StorageBackedDbWrite,
        change_request::{
            AssessChangeRequestDto, AssignChangeRequestTaskDto, ChangeRequestDeclineReason, ChangeRequestDetailDto,
            ChangeRequestDetailRow, ChangeRequestOrigin, ChangeRequestSearchField, ChangeRequestStatus,
            ChangeRequestSummaryDto, ClientApproveChangeRequestDto, ClientCancelChangeRequestDto,
            ClientChangeRequestDetailDto, ClientChangeRequestDetailRow, ClientChangeRequestSearchField,
            ClientChangeRequestSummaryDto, ClientCreateChangeRequestDto, ClientDeclineChangeRequestDto,
            ClientUpdateChangeRequestDto, CreateChangeRequestDto, UpdateChangeRequestDto,
        },
        project::ProjectStatus,
        query::QueryPayload,
    },
    entities::{
        change_requests::{self, Model as ChangeRequestModel},
        clients,
        prelude::{ChangeRequests, Clients, Projects, Tasks},
        projects, tasks,
    },
    error::AppError,
    services::attachments::AttachmentService,
    services::scope::ScopeService,
    utils::{
        multipart::{
            MarkdownUploadedFile, replace_markdown_attachment_urls, replace_markdown_attachment_urls_required,
        },
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub struct ChangeRequestService;

impl ChangeRequestService {
    async fn persist(
        db: &impl ConnectionTrait,
        active: change_requests::ActiveModel,
        expected: i32,
    ) -> Result<ChangeRequestModel, AppError> {
        let id = active.id.clone().unwrap();
        let result = ChangeRequests::update_many()
            .set(active)
            .filter(change_requests::Column::Id.eq(id))
            .filter(change_requests::Column::Version.eq(expected))
            .exec(db)
            .await?;
        if result.rows_affected != 1 {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_PROPOSAL_CHANGED.into()));
        }
        ChangeRequests::find_by_id(id)
            .one(db)
            .await?
            .ok_or(AppError::InternalError)
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateChangeRequestDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        Self::ensure_project_client_pair(db, user_id, dto.project_id, dto.client_id).await?;
        Self::ensure_project_accepts_new_change_requests(db, user_id, dto.project_id).await?;
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

        let created = change_request.insert(db).await.map_err(AppError::from)?;
        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn create_with_markdown_files(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        dto: CreateChangeRequestDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<StorageBackedDbWrite<ChangeRequestDetailDto>, AppError> {
        if files.is_empty() {
            return Ok(StorageBackedDbWrite::without_storage(
                Self::create(db, user_id, dto).await?,
            ));
        }

        Self::ensure_project_client_pair(db, user_id, dto.project_id, dto.client_id).await?;
        Self::ensure_project_accepts_new_change_requests(db, user_id, dto.project_id).await?;
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

        let created = change_request.insert(db).await.map_err(AppError::from)?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            db,
            storage,
            user_id,
            Some(created.project_id),
            None,
            None,
            Some(created.id),
            files,
        )
        .await?;

        let next_description =
            replace_markdown_attachment_urls_required(created.client_request_description.clone(), &upload.replacements);
        let expected_version = created.version;
        let mut active_model: change_requests::ActiveModel = created.into();
        active_model.client_request_description = Set(next_description);
        let updated = match Self::persist(db, active_model, expected_version).await {
            Ok(updated) => updated,
            Err(error) => {
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        Ok(StorageBackedDbWrite {
            value: Self::get_by_id(db, user_id, updated.id).await?,
            storage_keys: upload.storage_keys,
        })
    }

    pub async fn create_by_client(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        dto: ClientCreateChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        Self::ensure_client_project_ownership(db, user_id, client_id, dto.project_id).await?;
        Self::ensure_project_accepts_new_change_requests(db, user_id, dto.project_id).await?;

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

        let created = change_request.insert(db).await.map_err(AppError::from)?;
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
            .column(change_requests::Column::DeclineReason)
            .column(change_requests::Column::DeclineComment)
            .column(change_requests::Column::ClientApprovedAt)
            .column(change_requests::Column::ClientSignatureIp)
            .column(change_requests::Column::CreatedAt)
            .column(change_requests::Column::UpdatedAt)
            .column(change_requests::Column::Version)
            .column_as(projects::Column::Name, "project_name")
            .column_as(clients::Column::Name, "client_name")
            .into_model::<ChangeRequestDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_NOT_FOUND.into()))?;

        Ok(Self::map_detail_row(row)?)
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
            .column(change_requests::Column::Origin)
            .column(change_requests::Column::Status)
            .column(change_requests::Column::DeclineReason)
            .column(change_requests::Column::DeclineComment)
            .column(change_requests::Column::ClientApprovedAt)
            .column(change_requests::Column::ClientSignatureIp)
            .column(change_requests::Column::CreatedAt)
            .column(change_requests::Column::UpdatedAt)
            .column(change_requests::Column::Version)
            .column_as(projects::Column::Name, "project_name")
            .into_model::<ClientChangeRequestDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_NOT_FOUND.into()))?;

        Ok(Self::map_client_detail_row(row)?)
    }

    async fn update_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChangeRequestDto,
    ) -> Result<Uuid, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;
        let origin = Self::parse_origin(&model.origin)?;

        if model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_UPDATED_BY_CLIENT.into()));
        }

        if status == ChangeRequestStatus::Approved {
            let non_task_fields_changed = dto.project_id != model.project_id
                || dto.client_id != model.client_id
                || dto.title != model.title
                || dto.client_request_description != model.client_request_description
                || dto.technical_assessment != model.technical_assessment
                || dto.impact_hours != model.impact_hours
                || dto.impact_cost != model.impact_cost
                || dto.schedule_impact_days != model.schedule_impact_days;

            if non_task_fields_changed {
                return Err(AppError::Conflict(codes::CHANGE_REQUEST_APPROVED_TASK_ONLY.into()));
            }

            Self::ensure_task_ownership(db, user_id, model.project_id, dto.task_id).await?;

            let expected_version = model.version;
            let mut active_model: change_requests::ActiveModel = model.into();
            active_model.task_id = Set(dto.task_id);

            let updated = Self::persist(db, active_model, expected_version).await?;
            return Ok(updated.id);
        }

        if origin == ChangeRequestOrigin::ClientPortal {
            return Err(AppError::Forbidden(codes::CHANGE_REQUEST_CLIENT_OWNED.into()));
        }

        if !matches!(
            status,
            ChangeRequestStatus::Draft | ChangeRequestStatus::Submitted | ChangeRequestStatus::Assessed
        ) {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_FINALIZED_NOT_EDITABLE.into()));
        }

        Self::ensure_project_client_pair(db, user_id, dto.project_id, dto.client_id).await?;
        Self::ensure_task_ownership(db, user_id, dto.project_id, dto.task_id).await?;

        let expected_version = model.version;
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

        let updated = Self::persist(db, active_model, expected_version).await?;
        Ok(updated.id)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateChangeRequestDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let updated_id = Self::update_with_connection(db, user_id, id, dto).await?;
        Self::get_by_id(db, user_id, updated_id).await
    }

    pub async fn update_with_markdown_files(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        user_id: Uuid,
        id: Uuid,
        mut dto: UpdateChangeRequestDto,
        files: Vec<MarkdownUploadedFile>,
    ) -> Result<StorageBackedDbWrite<ChangeRequestDetailDto>, AppError> {
        if files.is_empty() {
            return Ok(StorageBackedDbWrite::without_storage(
                Self::update(db, user_id, id, dto).await?,
            ));
        }

        let model = Self::get_model_by_user(db, user_id, id).await?;
        let upload = AttachmentService::create_markdown_files_with_tracking_in_tx(
            db,
            storage,
            user_id,
            Some(model.project_id),
            None,
            None,
            Some(model.id),
            files,
        )
        .await?;

        dto.client_request_description =
            replace_markdown_attachment_urls_required(dto.client_request_description, &upload.replacements);
        dto.technical_assessment = replace_markdown_attachment_urls(dto.technical_assessment, &upload.replacements);
        let updated_id = match Self::update_with_connection(db, user_id, id, dto).await {
            Ok(updated_id) => updated_id,
            Err(error) => {
                AttachmentService::cleanup_storage_objects(storage, &upload.storage_keys).await;
                return Err(error);
            }
        };

        Ok(StorageBackedDbWrite {
            value: Self::get_by_id(db, user_id, updated_id).await?,
            storage_keys: upload.storage_keys,
        })
    }

    pub async fn client_update_spec(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
        dto: ClientUpdateChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_client(db, user_id, client_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_UPDATED_BY_FREELANCER.into()));
        }

        if !matches!(status, ChangeRequestStatus::Draft | ChangeRequestStatus::Submitted) {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_NOT_EDITABLE.into()));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.title = Set(dto.title);
        active_model.client_request_description = Set(dto.client_request_description);

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn assign_task_for_approved(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: AssignChangeRequestTaskDto,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_UPDATED_BY_CLIENT.into()));
        }

        if status != ChangeRequestStatus::Approved {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_TASK_REQUIRES_APPROVAL.into()));
        }

        let task_id = dto
            .task_id
            .ok_or_else(|| AppError::Conflict(codes::CHANGE_REQUEST_ORIGIN_REQUIRED.into()))?;
        ScopeService::classify(
            db,
            user_id,
            user_id,
            task_id,
            ClassifyTaskDto {
                change_request_id: Some(id),
                reason: "Trabajo adicional del cambio aprobado".into(),
            },
        )
        .await?;
        Self::get_by_id(db, user_id, id).await
    }

    pub async fn submit(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        version: i32,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        if model.version != version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_REVISION_CHANGED.into()));
        }
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Draft {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_SUBMIT_REQUIRES_DRAFT.into()));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Submitted.to_string());

        let updated = Self::persist(db, active_model, expected_version).await?;
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

        if !matches!(status, ChangeRequestStatus::Submitted | ChangeRequestStatus::Assessed)
            || model.version != dto.version
        {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_REVISION_NOT_ASSESSABLE.into()));
        }
        db.execute_unprepared("SELECT set_config('horbyta.publish_offer','true',true)")
            .await?;

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.target_milestone_id = Set(dto.target_milestone_id);
        active_model.technical_assessment = Set(Some(dto.technical_assessment));
        active_model.impact_hours = Set(dto.impact_hours);
        active_model.impact_cost = Set(dto.impact_cost);
        active_model.schedule_impact_days = Set(dto.schedule_impact_days);
        active_model.status = Set(ChangeRequestStatus::Assessed.to_string());

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    /// Reabre un cambio rechazado para negociar nuevas condiciones. El rechazo, su
    /// motivo y explicación permanecen en la versión anterior del historial.
    pub async fn reopen(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        version: i32,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        if model.version != version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_REVISION_CHANGED.into()));
        }
        if Self::parse_status(&model.status)? != ChangeRequestStatus::Declined {
            return Err(AppError::Conflict(
                codes::CHANGE_REQUEST_REOPEN_REQUIRES_DECLINED.into(),
            ));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Submitted.to_string());
        active_model.decline_reason = Set(None);
        active_model.decline_comment = Set(None);

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn mark_implemented(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        version: i32,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        if model.version != version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_REVISION_CHANGED.into()));
        }
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Approved {
            return Err(AppError::Conflict(
                codes::CHANGE_REQUEST_IMPLEMENT_REQUIRES_APPROVAL.into(),
            ));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Implemented.to_string());

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn cancel(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        version: i32,
    ) -> Result<ChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        if model.version != version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_REVISION_CHANGED.into()));
        }
        let status = Self::parse_status(&model.status)?;
        let origin = Self::parse_origin(&model.origin)?;

        if origin != ChangeRequestOrigin::FreelancerDashboard {
            return Err(AppError::Forbidden(codes::CHANGE_REQUEST_CANCEL_FREELANCER_ONLY.into()));
        }

        if matches!(
            status,
            ChangeRequestStatus::Approved
                | ChangeRequestStatus::Declined
                | ChangeRequestStatus::Implemented
                | ChangeRequestStatus::Cancelled
        ) {
            return Err(AppError::Conflict(
                codes::CHANGE_REQUEST_FINALIZED_NOT_CANCELLABLE.into(),
            ));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Cancelled.to_string());

        let updated = Self::persist(db, active_model, expected_version).await?;
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

        if matches!(status, ChangeRequestStatus::Approved | ChangeRequestStatus::Implemented) {
            let same = db
                .query_one(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "SELECT 1 AS ok FROM scope_effects WHERE change_request_id=$1 AND offer_version=$2",
                    [id.into(), dto.version.into()],
                ))
                .await?
                .is_some();
            if same {
                return Self::get_client_by_id(db, user_id, client_id, id).await;
            }
        }
        if status != ChangeRequestStatus::Assessed || model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_OFFER_CHANGED.into()));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Approved.to_string());
        active_model.client_approved_at = Set(Some(Self::now_with_offset()));
        active_model.client_signature_ip = Set(dto.client_signature_ip);

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn client_decline(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
        dto: ClientDeclineChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let comment = dto
            .decline_comment
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned);
        if dto.decline_reason == ChangeRequestDeclineReason::Other && comment.is_none() {
            return Err(AppError::BadRequest(
                codes::CHANGE_REQUEST_DECLINE_COMMENT_REQUIRED.into(),
            ));
        }
        let model = Self::get_model_by_client(db, user_id, client_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status == ChangeRequestStatus::Declined
            && model.version == dto.version + 1
            && model.decline_reason.as_deref() == Some(dto.decline_reason.to_string().as_str())
            && model.decline_comment == comment
        {
            return Self::get_client_by_id(db, user_id, client_id, id).await;
        }
        if status != ChangeRequestStatus::Assessed || model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_OFFER_CHANGED.into()));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Declined.to_string());
        active_model.client_approved_at = Set(None);
        active_model.client_signature_ip = Set(None);
        active_model.decline_reason = Set(Some(dto.decline_reason.to_string()));
        active_model.decline_comment = Set(comment);

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn client_cancel(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        id: Uuid,
        dto: ClientCancelChangeRequestDto,
    ) -> Result<ClientChangeRequestDetailDto, AppError> {
        let model = Self::get_model_by_client(db, user_id, client_id, id).await?;
        let status = Self::parse_status(&model.status)?;
        let origin = Self::parse_origin(&model.origin)?;

        if model.version != dto.version {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_UPDATED_BY_FREELANCER.into()));
        }

        if origin != ChangeRequestOrigin::ClientPortal {
            return Err(AppError::Forbidden(codes::CHANGE_REQUEST_CANCEL_CLIENT_ONLY.into()));
        }

        if status == ChangeRequestStatus::Assessed {
            return Err(AppError::Conflict(codes::CHANGE_REQUEST_ASSESSED_DECIDE_ONLY.into()));
        }

        if !matches!(status, ChangeRequestStatus::Draft | ChangeRequestStatus::Submitted) {
            return Err(AppError::Conflict(
                codes::CHANGE_REQUEST_FINALIZED_NOT_CANCELLABLE.into(),
            ));
        }

        let expected_version = model.version;
        let mut active_model: change_requests::ActiveModel = model.into();
        active_model.status = Set(ChangeRequestStatus::Cancelled.to_string());

        let updated = Self::persist(db, active_model, expected_version).await?;
        Self::get_client_by_id(db, user_id, client_id, updated.id).await
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let model = Self::get_model_by_user(db, user_id, id).await?;
        let status = Self::parse_status(&model.status)?;

        if status != ChangeRequestStatus::Cancelled {
            return Err(AppError::Conflict(
                codes::CHANGE_REQUEST_DELETE_REQUIRES_CANCELLED.into(),
            ));
        }

        change_requests::ActiveModel {
            id: Set(model.id),
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
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_NOT_FOUND.into()))?;

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
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_NOT_FOUND.into()))?;

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
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_PROJECT_FORBIDDEN.into()))?;

        let client_exists = Clients::find()
            .filter(clients::Column::Id.eq(client_id))
            .filter(clients::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .is_some();

        if !client_exists {
            return Err(AppError::NotFound(codes::CHANGE_REQUEST_CLIENT_FORBIDDEN.into()));
        }

        if project.client_id != Some(client_id) {
            return Err(AppError::BadRequest(codes::PROJECT_NOT_ASSIGNED_TO_CLIENT.into()));
        }

        Ok(())
    }

    async fn ensure_client_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .filter(projects::Column::ClientId.eq(client_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_PROJECT_FORBIDDEN.into()))?;

        Ok(())
    }

    async fn ensure_project_accepts_new_change_requests(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::CHANGE_REQUEST_PROJECT_FORBIDDEN.into()))?;

        if project.status == ProjectStatus::Closed.to_string() {
            return Err(AppError::Conflict(codes::PROJECT_CLOSED_NO_CHANGE_REQUESTS.into()));
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
                .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_TASK_FORBIDDEN.into()))?;

            if task.project_id != Some(project_id) {
                return Err(AppError::BadRequest(codes::TASK_NOT_IN_PROJECT.into()));
            }
        }

        Ok(())
    }

    fn parse_status(raw: &str) -> Result<ChangeRequestStatus, AppError> {
        ChangeRequestStatus::try_from(raw.to_string())
            .map_err(|_| AppError::Internal(format!("invalid status stored in change_requests: {}", raw)))
    }

    fn parse_origin(raw: &str) -> Result<ChangeRequestOrigin, AppError> {
        ChangeRequestOrigin::try_from(raw.to_string())
            .map_err(|_| AppError::Internal(format!("invalid origin stored in change_requests: {}", raw)))
    }

    fn now_with_offset() -> DateTime<FixedOffset> {
        Utc::now().fixed_offset()
    }

    fn map_detail_row(row: ChangeRequestDetailRow) -> Result<ChangeRequestDetailDto, AppError> {
        let decline_reason = match row.decline_reason {
            Some(value) => Some(
                ChangeRequestDeclineReason::from_str(&value)
                    .map_err(|e| AppError::Internal(format!("invalid decline reason: {}", e)))?,
            ),
            None => None,
        };

        Ok(ChangeRequestDetailDto {
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
            decline_reason,
            decline_comment: row.decline_comment,
            client_approved_at: row.client_approved_at,
            client_signature_ip: row.client_signature_ip,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: row.version,
        })
    }

    fn map_client_detail_row(row: ClientChangeRequestDetailRow) -> Result<ClientChangeRequestDetailDto, AppError> {
        let decline_reason = match row.decline_reason {
            Some(value) => Some(
                ChangeRequestDeclineReason::from_str(&value)
                    .map_err(|e| AppError::Internal(format!("invalid decline reason: {}", e)))?,
            ),
            None => None,
        };

        Ok(ClientChangeRequestDetailDto {
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
            origin: row.origin,
            status: row.status,
            decline_reason,
            decline_comment: row.decline_comment,
            client_approved_at: row.client_approved_at,
            client_signature_ip: row.client_signature_ip,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: row.version,
        })
    }
}
