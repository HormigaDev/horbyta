//! Entrega diferida de decisiones; la notificación y su confirmación son atómicas.
use crate::{
    entities::notifications::Model as Notification,
    error::AppError,
    services::{change_requests::ChangeRequestService, notifications::NotificationService},
    state::AppState,
    utils::notifications::publish_notification,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use uuid::Uuid;

pub async fn deliver_one(db: &DatabaseConnection) -> Result<Option<(Uuid, Option<Notification>)>, AppError> {
    let tx = db.begin().await?;
    let row=tx.query_one(Statement::from_string(DbBackend::Postgres,
      "SELECT d.event_id,e.change_request_id,e.kind,e.version,c.user_id,c.client_id FROM scope_delivery d JOIN scope_events e ON e.id=d.event_id JOIN change_requests c ON c.id=e.change_request_id WHERE d.delivered_at IS NULL AND d.next_attempt_at<=now() ORDER BY d.next_attempt_at FOR UPDATE OF d SKIP LOCKED LIMIT 1".to_owned())).await?;
    let Some(row) = row else {
        tx.rollback().await?;
        return Ok(None);
    };
    let event_id: Uuid = row.try_get("", "event_id")?;
    let owner: Uuid = row.try_get("", "user_id")?;
    let client: Uuid = row.try_get("", "client_id")?;
    let change: Uuid = row.try_get("", "change_request_id")?;
    let approved = row.try_get::<String>("", "kind")? == "approved";
    let result = async {
        let mut detail = ChangeRequestService::get_client_by_id(&tx, owner, client, change).await?;
        detail.status = if approved {
            crate::dtos::change_request::ChangeRequestStatus::Approved
        } else {
            crate::dtos::change_request::ChangeRequestStatus::Declined
        };
        detail.version = row.try_get("", "version")?;
        NotificationService::notify_client_change_request_decision(&tx, owner, client, &detail, approved).await
    }
    .await;
    match result {
        Ok(notification) => {
            tx.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE scope_delivery SET delivered_at=now(),attempts=attempts+1,last_error=NULL WHERE event_id=$1",
                [event_id.into()],
            ))
            .await?;
            tx.commit().await?;
            Ok(Some((owner, notification)))
        }
        Err(error) => {
            tx.rollback().await?;
            db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
                "UPDATE scope_delivery SET attempts=attempts+1,next_attempt_at=now()+interval '30 seconds',last_error='Error de entrega; se reintentará' WHERE event_id=$1 AND delivered_at IS NULL",[event_id.into()])).await?;
            Err(error)
        }
    }
}
pub async fn run(state: AppState) {
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        for _ in 0..10 {
            match deliver_one(state.db.as_ref()).await {
                Ok(Some((owner, notification))) => publish_notification(&state, owner, notification),
                Ok(None) => break,
                Err(error) => {
                    tracing::warn!(?error, "Se reintentará la entrega de una decisión de alcance");
                    break;
                }
            }
        }
    }
}
