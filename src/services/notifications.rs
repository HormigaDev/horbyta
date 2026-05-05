use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, sea_query::Expr};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    dtos::{
        change_request::ClientChangeRequestDetailDto, invoice::InvoiceDetailDto, milestone::MilestoneStatus,
        payment::PaymentDetailDto, user::UserPreferences,
    },
    entities::{
        notifications::{self, Model as NotificationModel},
        prelude::{Notifications, Users},
    },
    error::AppError,
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub user_id: Uuid,
    pub notification: NotificationModel,
}

pub struct CreateNotificationInput {
    pub user_id: Uuid,
    pub preference_key: &'static str,
    pub title: String,
    pub body: Option<String>,
    pub payload: Option<Value>,
    pub severity: &'static str,
    pub source: &'static str,
    pub entity_type: Option<&'static str>,
    pub entity_id: Option<Uuid>,
}

pub struct NotificationService;

impl NotificationService {
    pub async fn create_and_publish(
        state: &AppState,
        input: CreateNotificationInput,
    ) -> Result<Option<NotificationModel>, AppError> {
        if !Self::preference_enabled(state.db.as_ref(), input.user_id, input.preference_key).await? {
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
        .insert(state.db.as_ref())
        .await
        .map_err(AppError::Database)?;

        let _ = state.notification_bus.send(NotificationEvent {
            user_id: input.user_id,
            notification: notification.clone(),
        });

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
        state: &AppState,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<(), AppError> {
        let title = "New change request".to_string();
        let body = Some(format!(
            "The client created request #{}: {}",
            change_request.record_number, change_request.title
        ));

        Self::create_and_publish(
            state,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_created",
                title,
                body,
                "info",
            ),
        )
        .await?;

        Ok(())
    }

    pub async fn notify_client_change_request_updated(
        state: &AppState,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<(), AppError> {
        let title = "Change request updated".to_string();
        let body = Some(format!(
            "The client updated request #{}: {}",
            change_request.record_number, change_request.title
        ));

        Self::create_and_publish(
            state,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_updated",
                title,
                body,
                "info",
            ),
        )
        .await?;

        Ok(())
    }

    pub async fn notify_client_change_request_decision(
        state: &AppState,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
        approved: bool,
    ) -> Result<(), AppError> {
        let preference_key = if approved {
            "client_portal.change_request_approved"
        } else {
            "client_portal.change_request_declined"
        };
        let title = if approved {
            "Change request approved"
        } else {
            "Change request declined"
        }
        .to_string();
        let body = Some(format!(
            "The client {} request #{}: {}",
            if approved { "approved" } else { "declined" },
            change_request.record_number,
            change_request.title
        ));

        Self::create_and_publish(
            state,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                preference_key,
                title,
                body,
                if approved { "positive" } else { "warning" },
            ),
        )
        .await?;

        Ok(())
    }

    pub async fn notify_client_change_request_cancelled(
        state: &AppState,
        user_id: Uuid,
        client_id: Uuid,
        change_request: &ClientChangeRequestDetailDto,
    ) -> Result<(), AppError> {
        let title = "Change request cancelled".to_string();
        let body = Some(format!(
            "The client cancelled request #{}: {}",
            change_request.record_number, change_request.title
        ));

        Self::create_and_publish(
            state,
            Self::change_request_input(
                user_id,
                client_id,
                change_request,
                "client_portal.change_request_cancelled",
                title,
                body,
                "warning",
            ),
        )
        .await?;

        Ok(())
    }

    pub async fn notify_invoice_opened(state: &AppState, invoice: &InvoiceDetailDto) -> Result<(), AppError> {
        Self::create_and_publish(
            state,
            CreateNotificationInput {
                user_id: invoice.user_id,
                preference_key: "billing.invoice_opened",
                title: format!("Invoice #{} issued", invoice.record_number),
                body: Some(format!(
                    "The invoice for {} {} was marked as open.",
                    invoice.amount, invoice.currency_type
                )),
                payload: Some(json!({
                    "route": format!("/invoices/{}", invoice.id),
                    "invoiceId": invoice.id,
                    "invoiceRecordNumber": invoice.record_number,
                    "projectId": invoice.project_id,
                    "projectName": invoice.project_name,
                })),
                severity: "info",
                source: "invoice",
                entity_type: Some("invoice"),
                entity_id: Some(invoice.id),
            },
        )
        .await?;

        Ok(())
    }

    pub async fn notify_invoice_paid(state: &AppState, invoice: &InvoiceDetailDto) -> Result<(), AppError> {
        Self::create_and_publish(
            state,
            CreateNotificationInput {
                user_id: invoice.user_id,
                preference_key: "billing.invoice_paid",
                title: format!("Invoice #{} paid", invoice.record_number),
                body: Some(format!(
                    "The invoice for {} {} was marked as paid.",
                    invoice.amount, invoice.currency_type
                )),
                payload: Some(json!({
                    "route": format!("/invoices/{}", invoice.id),
                    "invoiceId": invoice.id,
                    "invoiceRecordNumber": invoice.record_number,
                    "projectId": invoice.project_id,
                    "projectName": invoice.project_name,
                })),
                severity: "positive",
                source: "invoice",
                entity_type: Some("invoice"),
                entity_id: Some(invoice.id),
            },
        )
        .await?;

        Ok(())
    }

    pub async fn notify_payment_received(
        state: &AppState,
        user_id: Uuid,
        payment: &PaymentDetailDto,
    ) -> Result<(), AppError> {
        Self::create_and_publish(
            state,
            CreateNotificationInput {
                user_id,
                preference_key: "billing.payment_received",
                title: format!("Payment #{} recorded", payment.record_number),
                body: Some(format!(
                    "A payment of {} {} was recorded.",
                    payment.amount, payment.currency_type
                )),
                payload: Some(json!({
                    "route": format!("/payments/{}", payment.id),
                    "paymentId": payment.id,
                    "paymentRecordNumber": payment.record_number,
                    "projectId": payment.project_id,
                    "projectName": payment.project_name,
                })),
                severity: "positive",
                source: "payment",
                entity_type: Some("payment"),
                entity_id: Some(payment.id),
            },
        )
        .await?;

        Ok(())
    }

    pub async fn notify_milestone_released(
        state: &AppState,
        user_id: Uuid,
        milestone: &crate::entities::milestones::Model,
    ) -> Result<(), AppError> {
        if milestone.status != MilestoneStatus::Released.to_string() {
            return Ok(());
        }

        Self::create_and_publish(
            state,
            CreateNotificationInput {
                user_id,
                preference_key: "work.milestone_released",
                title: "Milestone released".to_string(),
                body: Some(format!("Milestone \"{}\" was marked as released.", milestone.name)),
                payload: Some(json!({
                    "route": format!("/milestones/{}", milestone.id),
                    "milestoneId": milestone.id,
                    "projectId": milestone.project_id,
                    "name": milestone.name,
                })),
                severity: "positive",
                source: "milestone",
                entity_type: Some("milestone"),
                entity_id: Some(milestone.id),
            },
        )
        .await?;

        Ok(())
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
        title: String,
        body: Option<String>,
        severity: &'static str,
    ) -> CreateNotificationInput {
        CreateNotificationInput {
            user_id,
            preference_key,
            title,
            body,
            payload: Some(json!({
                "route": format!("/change-requests/{}", change_request.id),
                "changeRequestId": change_request.id,
                "changeRequestRecordNumber": change_request.record_number,
                "clientId": client_id,
                "projectId": change_request.project_id,
                "projectName": change_request.project_name,
                "title": change_request.title,
                "status": change_request.status,
            })),
            severity,
            source: "client_portal",
            entity_type: Some("change_request"),
            entity_id: Some(change_request.id),
        }
    }
}
