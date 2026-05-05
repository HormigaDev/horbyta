use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QuerySelect, RelationTrait, Set, TransactionTrait, prelude::Decimal,
};
use uuid::Uuid;

use crate::{
    dtos::{
        invoice::{InvoiceStatus, UpdateInvoiceStatusDto},
        payment::{
            CreatePaymentDto, CreateRefundDto, ParentPaymentReferenceDto, PaymentDetailDto, PaymentDetailRow,
            PaymentInvoiceReferenceDto, PaymentSearchField, PaymentSummaryDto, PaymentTransactionType,
            RelatedRefundDto, UpdatePaymentDto,
        },
        query::QueryPayload,
    },
    entities::{
        clients, invoices,
        payments::{self, Model as PaymentModel},
        prelude::{Invoices, Payments, Projects},
        project_billing, projects,
    },
    error::AppError,
    services::invoices::InvoiceService,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub struct PaymentService;

impl PaymentService {
    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<PaymentSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(payments::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<PaymentSearchField>()
                .ok()
                .map(PaymentSearchField::filter_field)
        })?;

        let query = Payments::find()
            .filter(condition)
            .join(JoinType::LeftJoin, payments::Relation::Projects.def())
            .join(JoinType::LeftJoin, projects::Relation::Clients.def())
            .join(JoinType::LeftJoin, payments::Relation::Invoices.def());

        let query = apply_ordering(
            query,
            &payload,
            PaymentSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<PaymentSearchField>()
                    .ok()
                    .map(PaymentSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let items = apply_pagination(
            query
                .select_only()
                .column(payments::Column::Id)
                .column(payments::Column::RecordNumber)
                .column(payments::Column::Amount)
                .column(payments::Column::CurrencyType)
                .column(payments::Column::TransactionType)
                .column(payments::Column::PaymentDate)
                .column(payments::Column::PaymentMethod)
                .column(payments::Column::Provider)
                .column(payments::Column::CreatedAt)
                .column(payments::Column::UpdatedAt)
                .column_as(projects::Column::Name, "project_name")
                .column_as(clients::Column::Name, "client_name")
                .column_as(invoices::Column::RecordNumber, "invoice_record_number"),
            &payload,
        )
        .into_model::<PaymentSummaryDto>()
        .all(db)
        .await?;

        Ok((items, total_items))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<PaymentDetailDto, AppError> {
        let row: PaymentDetailRow = Payments::find()
            .filter(payments::Column::Id.eq(id))
            .filter(payments::Column::UserId.eq(user_id))
            .join(JoinType::LeftJoin, payments::Relation::Projects.def())
            .join(JoinType::LeftJoin, projects::Relation::Clients.def())
            .select_only()
            .column(payments::Column::Id)
            .column(payments::Column::RecordNumber)
            .column(payments::Column::ProjectId)
            .column(payments::Column::InvoiceId)
            .column(payments::Column::ParentPaymentId)
            .column(payments::Column::Amount)
            .column(payments::Column::CurrencyType)
            .column(payments::Column::TransactionType)
            .column(payments::Column::PaymentDate)
            .column(payments::Column::PaymentMethod)
            .column(payments::Column::ReferenceCode)
            .column(payments::Column::Provider)
            .column(payments::Column::ProviderExternalId)
            .column(payments::Column::Notes)
            .column(payments::Column::CreatedAt)
            .column(payments::Column::UpdatedAt)
            .column_as(projects::Column::Name, "project_name")
            .column_as(projects::Column::ClientId, "client_id")
            .column_as(clients::Column::Name, "client_name")
            .into_model::<PaymentDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("payment not found".to_string()))?;

        let invoice = if let Some(invoice_id) = row.invoice_id {
            Self::get_invoice_reference(db, invoice_id).await?
        } else {
            None
        };

        let related_refunds = Self::get_related_refunds(db, id).await?;

        let parent_payment = if let Some(parent_id) = row.parent_payment_id {
            Self::get_parent_payment_reference(db, parent_id).await?
        } else {
            None
        };

        let status = Self::compute_status(&row.transaction_type, &related_refunds, row.amount);

        let transaction_type = PaymentTransactionType::try_from(row.transaction_type)
            .map_err(|_| AppError::Internal("invalid transaction type in database".to_string()))?;

        Ok(PaymentDetailDto {
            id: row.id,
            record_number: row.record_number,
            project_id: row.project_id,
            project_name: row.project_name.unwrap_or_default(),
            client_id: row.client_id,
            client_name: row.client_name,
            invoice,
            amount: row.amount,
            currency_type: row.currency_type,
            transaction_type,
            status,
            related_refunds,
            parent_payment,
            payment_date: row.payment_date,
            payment_method: row.payment_method,
            reference_code: row.reference_code,
            provider: row.provider,
            provider_external_id: row.provider_external_id,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn create(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        dto: CreatePaymentDto,
    ) -> Result<PaymentDetailDto, AppError> {
        let txn = db.begin().await.map_err(AppError::Database)?;

        Self::ensure_project_ownership(&txn, user_id, dto.project_id).await?;

        if let Some(invoice_id) = dto.invoice_id {
            Self::validate_invoice_for_project(&txn, user_id, invoice_id, dto.project_id).await?;
        }

        let currency_type = project_billing::Entity::find()
            .filter(project_billing::Column::ProjectId.eq(dto.project_id))
            .filter(project_billing::Column::UserId.eq(user_id))
            .one(&txn)
            .await
            .map_err(AppError::Database)?
            .map(|b| b.currency_type)
            .unwrap_or_else(|| "USD".to_string());

        let payment_date = dto.payment_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let payment = payments::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            invoice_id: Set(dto.invoice_id),
            parent_payment_id: Set(None),
            amount: Set(dto.amount),
            currency_type: Set(currency_type),
            transaction_type: Set("charge".to_string()),
            payment_date: Set(payment_date),
            provider: Set("manual".to_string()),
            provider_external_id: Set(None),
            payment_method: Set(dto.payment_method),
            reference_code: Set(dto.reference_code),
            notes: Set(dto.notes),
            ..Default::default()
        };

        let created = payment.insert(&txn).await.map_err(AppError::Database)?;

        if let Some(invoice_id) = dto.invoice_id {
            InvoiceService::update_status(
                &txn,
                user_id,
                invoice_id,
                UpdateInvoiceStatusDto {
                    status: InvoiceStatus::Paid,
                },
            )
            .await?;
        }

        txn.commit().await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn create_refund(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateRefundDto,
    ) -> Result<PaymentDetailDto, AppError> {
        let parent = Self::find_by_id(db, user_id, dto.parent_payment_id).await?;

        if parent.transaction_type != "charge" {
            return Err(AppError::BadRequest(
                "only a charge-type payment can be refunded".to_string(),
            ));
        }

        let existing_refunds_total: Decimal = Payments::find()
            .filter(payments::Column::ParentPaymentId.eq(parent.id))
            .all(db)
            .await?
            .iter()
            .map(|r| r.amount)
            .sum();

        let remaining = parent.amount - existing_refunds_total;
        if dto.amount > remaining {
            return Err(AppError::BadRequest(format!(
                "refund amount ({}) exceeds the available balance ({})",
                dto.amount, remaining
            )));
        }

        let refund_date = dto.refund_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        let refund = payments::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(parent.project_id),
            invoice_id: Set(parent.invoice_id),
            parent_payment_id: Set(Some(parent.id)),
            amount: Set(dto.amount),
            currency_type: Set(parent.currency_type),
            transaction_type: Set("refund".to_string()),
            payment_date: Set(refund_date),
            provider: Set("manual".to_string()),
            provider_external_id: Set(None),
            payment_method: Set(None),
            reference_code: Set(None),
            notes: Set(dto.notes),
            ..Default::default()
        };

        let created = refund.insert(db).await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdatePaymentDto,
    ) -> Result<PaymentDetailDto, AppError> {
        let payment = Self::find_by_id(db, user_id, id).await?;

        if payment.provider != "manual" {
            return Err(AppError::Conflict(
                "only manually registered payments can be edited".to_string(),
            ));
        }

        let mut active: payments::ActiveModel = payment.into();
        active.payment_method = Set(dto.payment_method);
        active.reference_code = Set(dto.reference_code);
        active.notes = Set(dto.notes);
        active.update(db).await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, id).await
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let payment = Self::find_by_id(db, user_id, id).await?;

        if payment.provider != "manual" {
            return Err(AppError::Conflict(
                "only manually registered payments can be deleted".to_string(),
            ));
        }

        payments::ActiveModel {
            id: Set(payment.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<PaymentModel, AppError> {
        Payments::find()
            .filter(payments::Column::Id.eq(id))
            .filter(payments::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("payment not found".to_string()))
    }

    async fn ensure_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest("invalid project or no permissions".to_string()))?;
        Ok(())
    }

    async fn validate_invoice_for_project(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        invoice_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        let invoice = Invoices::find()
            .filter(invoices::Column::Id.eq(invoice_id))
            .filter(invoices::Column::UserId.eq(user_id))
            .filter(invoices::Column::ProjectId.eq(project_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest("invalid invoice or does not belong to the project".to_string()))?;

        if invoice.status != InvoiceStatus::Open.to_string() {
            return Err(AppError::Conflict(
                "payments can only be recorded for invoices in 'open' status".to_string(),
            ));
        }

        Ok(())
    }

    async fn get_invoice_reference(
        db: &impl ConnectionTrait,
        invoice_id: Uuid,
    ) -> Result<Option<PaymentInvoiceReferenceDto>, AppError> {
        let invoice = Invoices::find()
            .filter(invoices::Column::Id.eq(invoice_id))
            .one(db)
            .await?;

        Ok(invoice.map(|inv| PaymentInvoiceReferenceDto {
            id: inv.id,
            record_number: inv.record_number,
            status: inv.status,
        }))
    }

    async fn get_related_refunds(
        db: &impl ConnectionTrait,
        payment_id: Uuid,
    ) -> Result<Vec<RelatedRefundDto>, AppError> {
        let refunds = Payments::find()
            .filter(payments::Column::ParentPaymentId.eq(payment_id))
            .all(db)
            .await?;

        Ok(refunds
            .into_iter()
            .map(|r| RelatedRefundDto {
                id: r.id,
                record_number: r.record_number,
                amount: r.amount,
                payment_date: r.payment_date,
                notes: r.notes,
            })
            .collect())
    }

    async fn get_parent_payment_reference(
        db: &impl ConnectionTrait,
        parent_id: Uuid,
    ) -> Result<Option<ParentPaymentReferenceDto>, AppError> {
        let parent = Payments::find()
            .filter(payments::Column::Id.eq(parent_id))
            .one(db)
            .await?;

        Ok(parent.map(|p| ParentPaymentReferenceDto {
            id: p.id,
            record_number: p.record_number,
            payment_date: p.payment_date,
        }))
    }

    fn compute_status(transaction_type: &str, related_refunds: &[RelatedRefundDto], amount: Decimal) -> String {
        if transaction_type == "refund" {
            return "refunded".to_string();
        }

        let total_refunded: Decimal = related_refunds.iter().map(|r| r.amount).sum();

        if total_refunded >= amount {
            "fully_refunded".to_string()
        } else if total_refunded > Decimal::ZERO {
            "partially_refunded".to_string()
        } else {
            "completed".to_string()
        }
    }
}
