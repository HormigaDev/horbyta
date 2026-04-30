use sea_orm::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    dtos::{
        project::{
            ClientProjectDetailDto, ClientProjectDetailRow, ClientProjectSearchField, ClientProjectSummaryDto,
            CreateProjectDto, ProjectDetailDto, ProjectDetailRow, ProjectSearchField, ProjectSummaryDto,
            UpdateProjectBillableDto, UpdateProjectCategoryDto, UpdateProjectDto, UpdateProjectStatusDto,
        },
        project_billing::{CreateProjectBillingDto, ProjectBillingType, UpdateProjectBillingDto},
        query::QueryPayload,
    },
    entities::{
        categories, clients, milestones,
        prelude::Projects,
        project_billing,
        projects::{self, Model as ProjectModel},
    },
    error::AppError,
    services::milestones::MilestoneService,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub struct ProjectService;

struct NormalizedProjectBilling {
    currency_type: String,
    billing_type: String,
    hourly_rate: Option<Decimal>,
    fixed_price: Option<Decimal>,
    retainer_amount: Option<Decimal>,
    payment_day: Option<i16>,
}

impl ProjectService {
    fn to_update_billing_dto(dto: CreateProjectBillingDto) -> UpdateProjectBillingDto {
        UpdateProjectBillingDto {
            currency_type: dto.currency_type,
            billing_type: dto.billing_type,
            hourly_rate: dto.hourly_rate,
            fixed_price: dto.fixed_price,
            retainer_amount: dto.retainer_amount,
            payment_day: dto.payment_day,
        }
    }

    fn normalize_billing_payload(dto: UpdateProjectBillingDto) -> Result<NormalizedProjectBilling, AppError> {
        let currency_type = dto.currency_type.trim().to_uppercase();
        let billing_type = dto.billing_type.to_string();

        match dto.billing_type {
            ProjectBillingType::Fixed => {
                if dto.hourly_rate.is_some() || dto.retainer_amount.is_some() || dto.payment_day.is_some() {
                    return Err(AppError::BadRequest(
                        "if billing type is fixed cant send hourly or retainer info".to_string(),
                    ));
                }

                let fixed_price = dto
                    .fixed_price
                    .ok_or_else(|| AppError::BadRequest("fixed price must be provided".to_string()))?;

                Ok(NormalizedProjectBilling {
                    currency_type,
                    billing_type,
                    hourly_rate: None,
                    fixed_price: Some(fixed_price),
                    retainer_amount: None,
                    payment_day: None,
                })
            }
            ProjectBillingType::Hourly => {
                if dto.fixed_price.is_some() || dto.retainer_amount.is_some() || dto.payment_day.is_some() {
                    return Err(AppError::BadRequest(
                        "if billing type is hourly cant send fixed price or retainer info".to_string(),
                    ));
                }

                let hourly_rate = dto
                    .hourly_rate
                    .ok_or_else(|| AppError::BadRequest("hourly rate must be provided".to_string()))?;

                Ok(NormalizedProjectBilling {
                    currency_type,
                    billing_type,
                    hourly_rate: Some(hourly_rate),
                    fixed_price: None,
                    retainer_amount: None,
                    payment_day: None,
                })
            }
            ProjectBillingType::Retainer => {
                if dto.hourly_rate.is_some() || dto.fixed_price.is_some() {
                    return Err(AppError::BadRequest(
                        "if billing type is retainer cant send hourly or fixed price info".to_string(),
                    ));
                }

                let retainer_amount = dto
                    .retainer_amount
                    .ok_or_else(|| AppError::BadRequest("retainer amount must be provided".to_string()))?;

                let payment_day = dto
                    .payment_day
                    .ok_or_else(|| AppError::BadRequest("payment day must be provided".to_string()))?;

                Ok(NormalizedProjectBilling {
                    currency_type,
                    billing_type,
                    hourly_rate: None,
                    fixed_price: None,
                    retainer_amount: Some(retainer_amount),
                    payment_day: Some(payment_day),
                })
            }
        }
    }

    async fn upsert_billing_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        billing: NormalizedProjectBilling,
    ) -> Result<project_billing::Model, AppError> {
        let existing = project_billing::Entity::find()
            .filter(project_billing::Column::UserId.eq(user_id))
            .filter(project_billing::Column::ProjectId.eq(project_id))
            .one(db)
            .await
            .map_err(AppError::Database)?;

        let result = match existing {
            Some(model) => {
                let mut active: project_billing::ActiveModel = model.into();
                active.currency_type = Set(billing.currency_type);
                active.billing_type = Set(billing.billing_type);
                active.hourly_rate = Set(billing.hourly_rate);
                active.fixed_price = Set(billing.fixed_price);
                active.retainer_amount = Set(billing.retainer_amount);
                active.payment_day = Set(billing.payment_day);

                active.update(db).await.map_err(AppError::Database)?
            }
            None => project_billing::ActiveModel {
                user_id: Set(user_id),
                project_id: Set(project_id),
                currency_type: Set(billing.currency_type),
                billing_type: Set(billing.billing_type),
                hourly_rate: Set(billing.hourly_rate),
                fixed_price: Set(billing.fixed_price),
                retainer_amount: Set(billing.retainer_amount),
                payment_day: Set(billing.payment_day),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(AppError::Database)?,
        };

        Ok(result)
    }

    async fn delete_billing_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        project_billing::Entity::delete_many()
            .filter(project_billing::Column::UserId.eq(user_id))
            .filter(project_billing::Column::ProjectId.eq(project_id))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    async fn create_with_connection(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateProjectDto,
    ) -> Result<ProjectModel, AppError> {
        if !dto.billable && dto.project_billing.is_some() {
            return Err(AppError::BadRequest(
                "cant send project_billing if project is not billable".to_string(),
            ));
        }

        let project = projects::ActiveModel {
            user_id: Set(user_id),
            name: Set(dto.name),
            description: Set(dto.description),
            category_id: Set(dto.category_id),
            client_id: Set(dto.client_id),
            billable: Set(dto.billable),
            ..Default::default()
        };

        let created_project = project.insert(db).await.map_err(AppError::Database)?;

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
                    .exec(db)
                    .await
                    .map_err(AppError::Database)?;
            }
        }

        if let Some(billing) = dto.project_billing {
            let normalized = Self::normalize_billing_payload(Self::to_update_billing_dto(billing))?;
            let _ = Self::upsert_billing_with_connection(db, user_id, created_project.id, normalized).await?;
        }

        Ok(created_project)
    }

    pub async fn create_in_tx(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateProjectDto,
    ) -> Result<ProjectModel, AppError> {
        Self::create_with_connection(db, user_id, dto).await
    }

    pub async fn create(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        dto: CreateProjectDto,
    ) -> Result<ProjectModel, AppError> {
        let txn = db.begin().await.map_err(AppError::Database)?;
        let created_project = Self::create_with_connection(&txn, user_id, dto).await?;

        txn.commit().await.map_err(AppError::Database)?;

        Ok(created_project)
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ProjectDetailDto, AppError> {
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
            .column(projects::Column::Billable)
            .column(projects::Column::CreatedAt)
            .column(projects::Column::UpdatedAt)
            .column_as(categories::Column::Name, "category_name")
            .column_as(clients::Column::Name, "client_name")
            .into_model::<ProjectDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

        let milestones = MilestoneService::list_project_milestones_preview(db, user_id, id).await?;
        let this_project_billing = project_billing::Entity::find()
            .filter(project_billing::Column::UserId.eq(user_id))
            .filter(project_billing::Column::ProjectId.eq(id))
            .one(db)
            .await
            .map_err(AppError::Database)?;

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
            billable: row.billable,
            milestones,
            project_billing: this_project_billing,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(project)
    }

    pub async fn get_client_project_by_id(
        db: &impl ConnectionTrait,
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
            .column(projects::Column::Billable)
            .column(projects::Column::CreatedAt)
            .column(projects::Column::UpdatedAt)
            .into_model::<ClientProjectDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

        let milestones =
            MilestoneService::list_client_project_milestones_preview(db, user_id, client_id, project_id).await?;

        let this_project_billing = project_billing::Entity::find()
            .filter(project_billing::Column::UserId.eq(user_id))
            .filter(project_billing::Column::ProjectId.eq(project_id))
            .one(db)
            .await
            .map_err(AppError::Database)?;

        Ok(ClientProjectDetailDto {
            id: row.id,
            record_number: row.record_number,
            name: row.name,
            description: row.description,
            status: row.status,
            billable: row.billable,
            milestones,
            project_billing: this_project_billing,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn update(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectDto,
    ) -> Result<ProjectModel, AppError> {
        if !dto.billable && dto.project_billing.is_some() {
            return Err(AppError::BadRequest(
                "cant send project_billing if project is not billable".to_string(),
            ));
        }

        let project = Self::find_by_id(db, user_id, id).await?;
        let txn = db.begin().await.map_err(AppError::Database)?;

        let mut active_project: projects::ActiveModel = project.into();
        active_project.name = Set(dto.name);
        active_project.description = Set(dto.description);
        active_project.category_id = Set(dto.category_id);
        active_project.client_id = Set(dto.client_id);
        active_project.status = Set(dto.status.to_string());
        active_project.billable = Set(dto.billable);

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

        if !dto.billable {
            Self::delete_billing_with_connection(&txn, user_id, updated_project.id).await?;
        } else if let Some(billing) = dto.project_billing {
            let normalized = Self::normalize_billing_payload(billing)?;
            let _ = Self::upsert_billing_with_connection(&txn, user_id, updated_project.id, normalized).await?;
        }

        txn.commit().await.map_err(AppError::Database)?;

        Ok(updated_project)
    }

    pub async fn update_billable(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectBillableDto,
    ) -> Result<ProjectModel, AppError> {
        let project = Self::find_by_id(db, user_id, id).await?;
        let txn = db.begin().await.map_err(AppError::Database)?;

        let mut active_project: projects::ActiveModel = project.into();
        active_project.billable = Set(dto.billable);

        let updated_project = active_project.update(&txn).await.map_err(AppError::Database)?;

        if !dto.billable {
            Self::delete_billing_with_connection(&txn, user_id, updated_project.id).await?;
        }

        txn.commit().await.map_err(AppError::Database)?;

        Ok(updated_project)
    }

    pub async fn upsert_billing(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectBillingDto,
    ) -> Result<project_billing::Model, AppError> {
        let project = Self::find_by_id(db, user_id, id).await?;
        let txn = db.begin().await.map_err(AppError::Database)?;

        if !project.billable {
            let mut active_project: projects::ActiveModel = project.into();
            active_project.billable = Set(true);
            let _ = active_project.update(&txn).await.map_err(AppError::Database)?;
        }

        let normalized = Self::normalize_billing_payload(dto)?;
        let updated_billing = Self::upsert_billing_with_connection(&txn, user_id, id, normalized).await?;

        txn.commit().await.map_err(AppError::Database)?;

        Ok(updated_billing)
    }

    pub async fn update_category(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectCategoryDto,
    ) -> Result<ProjectModel, AppError> {
        let project = Self::find_by_id(db, user_id, id).await?;
        let mut active_project: projects::ActiveModel = project.into();

        active_project.category_id = Set(dto.category_id);

        active_project.update(db).await.map_err(AppError::Database)
    }

    pub async fn update_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateProjectStatusDto,
    ) -> Result<ProjectModel, AppError> {
        let project = Self::find_by_id(db, user_id, id).await?;
        let mut active_project: projects::ActiveModel = project.into();

        active_project.status = Set(dto.status.to_string());

        active_project.update(db).await.map_err(AppError::Database)
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let project = Self::find_by_id(db, user_id, id).await?;

        projects::ActiveModel {
            id: Set(project.id),
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

        let total_items = query.clone().count(db).await?;

        let models = apply_pagination(
            query
                .left_join(categories::Entity)
                .left_join(clients::Entity)
                .select_only()
                .column(projects::Column::Id)
                .column(projects::Column::RecordNumber)
                .column(projects::Column::Name)
                .column(projects::Column::Status)
                .column(projects::Column::Billable)
                .column_as(categories::Column::Id, "category_id")
                .column_as(categories::Column::Name, "category_name")
                .column_as(clients::Column::Id, "client_id")
                .column_as(clients::Column::Name, "client_name")
                .column(projects::Column::CreatedAt)
                .column(projects::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ProjectSummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    pub async fn client_search_projects(
        db: &impl ConnectionTrait,
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

        let total_items = query.clone().count(db).await?;

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
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ProjectModel, AppError> {
        let project = Projects::find()
            .filter(projects::Column::Id.eq(id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

        Ok(project)
    }
}
