use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    dtos::{
        change_request::ClientChangeRequestDetailDto, invoice::InvoiceDetailDto, milestone::MilestoneStatus,
        notification::CreateNotificationInput, payment::PaymentDetailDto, user::UserPreferences,
    },
    entities::{
        notifications::{self, Model as NotificationModel},
        prelude::{Notifications, Users},
    },
    error::AppError,
};

pub struct NotificationService;

impl NotificationService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateNotificationInput,
    ) -> Result<Option<NotificationModel>, AppError> {
        if !Self::preference_enabled(db, input.user_id, input.preference_key).await? {
            return Ok(None);
        }

        let notification = notifications::ActiveModel {
            user_id: Set(input.user_id),
            preference_key: Set(input.preference_key.to_string()),
            event_type: Set(input.preference_key.to_string()),
            title: Set(input.title),
            body: Set(input.body),
            payload: Set(input.payload),
            severity: Set(input.severity.to_string()),
            source: Set(input.source.to_string()),
            entity_type: Set(input.entity_type.map(ToString::to_string)),
            entity_id: Set(input.entity_id),
            is_read: Set(false),
            read_at: Set(None),
            received_at: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(AppError::Database)?;

        Ok(Some(notification))
    }

    pub async fn mark_as_read(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let now = Utc::now().fixed_offset();

        Notifications::update_many()
            .col_expr(notifications::Column::IsRead, Expr::value(true))
            .col_expr(notifications::Column::ReadAt, Expr::value(now))
            .col_expr(notifications::Column::ReceivedAt, Expr::value(now))
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::Id.eq(id))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn mark_all_as_read(db: &impl ConnectionTrait, user_id: Uuid) -> Result<(), AppError> {
        let now = Utc::now().fixed_offset();

        Notifications::update_many()
            .col_expr(notifications::Column::IsRead, Expr::value(true))
            .col_expr(notifications::Column::ReadAt, Expr::value(now))
            .col_expr(notifications::Column::ReceivedAt, Expr::value(now))
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn delete_all(db: &impl ConnectionTrait, user_id: Uuid) -> Result<(), AppError> {
        Notifications::delete_many()
            .filter(notifications::Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn mark_as_received(db: &impl ConnectionTrait, user_id: Uuid, ids: Vec<Uuid>) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        Notifications::update_many()
            .col_expr(
                notifications::Column::ReceivedAt,
                Expr::value(Utc::now().fixed_offset()),
            )
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::Id.is_in(ids))
            .filter(notifications::Column::ReceivedAt.is_null())
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn mark_unread_as_received(db: &impl ConnectionTrait, user_id: Uuid) -> Result<(), AppError> {
        Notifications::update_many()
            .col_expr(
                notifications::Column::ReceivedAt,
                Expr::value(Utc::now().fixed_offset()),
            )
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .filter(notifications::Column::ReceivedAt.is_null())
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn notify_client_change_request_created(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_created",
                "notifications.clientPortal.changeRequestCreated.title",
                "notifications.clientPortal.changeRequestCreated.body",
                "info",
            ),
        )
        .await
    }

    pub async fn notify_client_change_request_updated(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_updated",
                "notifications.clientPortal.changeRequestUpdated.title",
                "notifications.clientPortal.changeRequestUpdated.body",
                "info",
            ),
        )
        .await
    }

    pub async fn notify_client_change_request_decision(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
        approved: bool,
    ) -> Result<Option<NotificationModel>, AppError> {
        let preference_key = if approved {
            "client_portal.change_request_approved"
        } else {
            "client_portal.change_request_declined"
        };
        let (title_key, body_key) = if approved {
            (
                "notifications.clientPortal.changeRequestApproved.title",
                "notifications.clientPortal.changeRequestApproved.body",
            )
        } else {
            (
                "notifications.clientPortal.changeRequestDeclined.title",
                "notifications.clientPortal.changeRequestDeclined.body",
            )
        };

        Self::create(
            db,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                preference_key,
                title_key,
                body_key,
                if approved { "positive" } else { "warning" },
            ),
        )
        .await
    }

    pub async fn notify_client_change_request_cancelled(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_cancelled",
                "notifications.clientPortal.changeRequestCancelled.title",
                "notifications.clientPortal.changeRequestCancelled.body",
                "warning",
            ),
        )
        .await
    }

    pub async fn notify_invoice_opened(
        db: &impl ConnectionTrait,
        invoice: &InvoiceDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            CreateNotificationInput {
                user_id: invoice.user_id,
                preference_key: "billing.invoice_opened",
                title: "notifications.billing.invoiceOpened.title".to_string(),
                body: Some("notifications.billing.invoiceOpened.body".to_string()),
                payload: Some(Self::translated_payload(
                    json!({
                        "route": format!("/invoices/{}", invoice.id),
                        "invoiceId": invoice.id,
                        "invoiceRecordNumber": invoice.record_number,
                        "projectId": invoice.project_id,
                        "projectName": invoice.project_name,
                    }),
                    "notifications.billing.invoiceOpened.title",
                    "notifications.billing.invoiceOpened.body",
                    json!({
                        "recordNumber": invoice.record_number,
                        "amount": invoice.amount,
                        "currency": invoice.currency_type,
                    }),
                )),
                severity: "info",
                source: "invoice",
                entity_type: Some("invoice"),
                entity_id: Some(invoice.id),
            },
        )
        .await
    }

    pub async fn notify_invoice_paid(
        db: &impl ConnectionTrait,
        invoice: &InvoiceDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            CreateNotificationInput {
                user_id: invoice.user_id,
                preference_key: "billing.invoice_paid",
                title: "notifications.billing.invoicePaid.title".to_string(),
                body: Some("notifications.billing.invoicePaid.body".to_string()),
                payload: Some(Self::translated_payload(
                    json!({
                        "route": format!("/invoices/{}", invoice.id),
                        "invoiceId": invoice.id,
                        "invoiceRecordNumber": invoice.record_number,
                        "projectId": invoice.project_id,
                        "projectName": invoice.project_name,
                    }),
                    "notifications.billing.invoicePaid.title",
                    "notifications.billing.invoicePaid.body",
                    json!({
                        "recordNumber": invoice.record_number,
                        "amount": invoice.amount,
                        "currency": invoice.currency_type,
                    }),
                )),
                severity: "positive",
                source: "invoice",
                entity_type: Some("invoice"),
                entity_id: Some(invoice.id),
            },
        )
        .await
    }

    pub async fn notify_payment_received(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payment: &PaymentDetailDto,
    ) -> Result<Option<NotificationModel>, AppError> {
        Self::create(
            db,
            CreateNotificationInput {
                user_id,
                preference_key: "billing.payment_received",
                title: "notifications.billing.paymentReceived.title".to_string(),
                body: Some("notifications.billing.paymentReceived.body".to_string()),
                payload: Some(Self::translated_payload(
                    json!({
                        "route": format!("/payments/{}", payment.id),
                        "paymentId": payment.id,
                        "paymentRecordNumber": payment.record_number,
                        "projectId": payment.project_id,
                        "projectName": payment.project_name,
                    }),
                    "notifications.billing.paymentReceived.title",
                    "notifications.billing.paymentReceived.body",
                    json!({
                        "recordNumber": payment.record_number,
                        "amount": payment.amount,
                        "currency": payment.currency_type,
                    }),
                )),
                severity: "positive",
                source: "payment",
                entity_type: Some("payment"),
                entity_id: Some(payment.id),
            },
        )
        .await
    }

    pub async fn notify_milestone_released(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        milestone: &crate::entities::milestones::Model,
    ) -> Result<Option<NotificationModel>, AppError> {
        if milestone.status != MilestoneStatus::Released.to_string() {
            return Ok(None);
        }

        Self::create(
            db,
            CreateNotificationInput {
                user_id,
                preference_key: "work.milestone_released",
                title: "notifications.work.milestoneReleased.title".to_string(),
                body: Some("notifications.work.milestoneReleased.body".to_string()),
                payload: Some(Self::translated_payload(
                    json!({
                        "route": format!("/milestones/{}", milestone.id),
                        "milestoneId": milestone.id,
                        "projectId": milestone.project_id,
                        "name": milestone.name,
                    }),
                    "notifications.work.milestoneReleased.title",
                    "notifications.work.milestoneReleased.body",
                    json!({
                        "name": milestone.name,
                    }),
                )),
                severity: "positive",
                source: "milestone",
                entity_type: Some("milestone"),
                entity_id: Some(milestone.id),
            },
        )
        .await
    }

    async fn preference_enabled(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        preference_key: &str,
    ) -> Result<bool, AppError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        let preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        Ok(preferences.notifications.is_enabled(preference_key))
    }

    fn change_request_input(
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
        preference_key: &'static str,
        title_key: &'static str,
        body_key: &'static str,
        severity: &'static str,
    ) -> CreateNotificationInput {
        CreateNotificationInput {
            user_id,
            preference_key,
            title: title_key.to_string(),
            body: Some(body_key.to_string()),
            payload: Some(Self::translated_payload(
                json!({
                    "route": format!("/change-requests/{}", change_request.id),
                    "changeRequestId": change_request.id,
                    "changeRequestRecordNumber": change_request.record_number,
                    "clientId": client_id,
                    "projectId": change_request.project_id,
                    "projectName": change_request.project_name,
                    "title": change_request.title,
                    "status": change_request.status,
                }),
                title_key,
                body_key,
                json!({
                    "recordNumber": change_request.record_number,
                    "title": change_request.title,
                }),
            )),
            severity,
            source: "client_portal",
            entity_type: Some("change_request"),
            entity_id: Some(change_request.id),
        }
    }

    fn translated_payload(mut payload: Value, title_key: &'static str, body_key: &'static str, params: Value) -> Value {
        if let Some(payload) = payload.as_object_mut() {
            payload.insert(
                "translation".to_string(),
                json!({
                    "titleKey": title_key,
                    "bodyKey": body_key,
                    "params": params,
                }),
            );
        }

        payload
    }
}
