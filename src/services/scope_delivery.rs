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
/// Conexión dedicada: LISTEN no ocupa un slot del pool de consultas ni consulta
/// periódicamente la cola. El canal se resuelve antes de procesar pendientes.
pub async fn listen(db: &DatabaseConnection) -> Result<sea_orm::sqlx::postgres::PgListener, AppError> {
    use sea_orm::sqlx::postgres::{PgListener, PgPoolOptions};
    let channel: String = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT 'horbyta_scope_delivery_'||md5(current_schema()) AS channel".to_owned(),
        ))
        .await?
        .ok_or(AppError::InternalError)?
        .try_get("", "channel")?;
    let options = db.get_postgres_connection_pool().connect_options();
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .min_connections(0)
        .idle_timeout(None)
        .max_lifetime(None)
        .test_before_acquire(false)
        .connect_lazy_with((*options).clone().application_name("horbyta-scope-delivery"));
    let mut listener = PgListener::connect_with(&pool).await.map_err(signal_error)?;
    listener.listen(&channel).await.map_err(signal_error)?;
    Ok(listener)
}
fn signal_error(error: sea_orm::sqlx::Error) -> AppError {
    AppError::Database(sea_orm::DbErr::Conn(sea_orm::RuntimeErr::SqlxError(error)))
}

/// Solo hay temporizador cuando existe una entrega pendiente, incluida una fila
/// bloqueada por otro consumidor. Una cola vacía queda esperando NOTIFY.
pub async fn next_attempt_delay(db: &DatabaseConnection) -> Result<Option<std::time::Duration>, AppError> {
    let row = db.query_one(Statement::from_string(DbBackend::Postgres,
        "SELECT extract(epoch FROM (min(next_attempt_at)-clock_timestamp()))::double precision AS delay FROM scope_delivery WHERE delivered_at IS NULL".to_owned()))
        .await?.ok_or(AppError::InternalError)?;
    let delay: Option<f64> = row.try_get("", "delay")?;
    Ok(delay.map(|seconds| std::time::Duration::from_secs_f64(seconds.clamp(0.1, 86_400.0))))
}

pub async fn run(state: AppState) {
    loop {
        match listen(state.db.as_ref()).await {
            Ok(mut listener) => {
                // Suscribir antes de recuperar: no se pierde una decisión que se
                // confirme entre la lectura inicial y el comienzo de la espera.
                loop {
                    let mut failed = false;
                    loop {
                        match deliver_one(state.db.as_ref()).await {
                            Ok(Some((owner, notification))) => {
                                publish_notification(&state, owner, notification);
                                tokio::task::yield_now().await;
                            }
                            Ok(None) => break,
                            Err(error) => {
                                tracing::warn!(?error, "Se reintentará la entrega de una decisión de alcance");
                                failed = true;
                                break;
                            }
                        }
                    }
                    let delay = match next_attempt_delay(state.db.as_ref()).await {
                        Ok(delay) => delay.or_else(|| failed.then_some(std::time::Duration::from_secs(30))),
                        Err(error) => {
                            tracing::warn!(?error, "No fue posible programar la entrega pendiente");
                            Some(std::time::Duration::from_secs(30))
                        }
                    };
                    let deadline = async {
                        match delay {
                            Some(delay) => tokio::time::sleep(delay).await,
                            None => std::future::pending::<()>().await,
                        }
                    };
                    tokio::select! {
                        signal = listener.try_recv() => match signal {
                            // None indica reconexión: recuperar los eventos que se
                            // confirmaron mientras no había escucha activa.
                            Ok(_) => {},
                            Err(error) => {
                                tracing::warn!(?error, "Se reconectará la escucha de decisiones de alcance");
                                break;
                            }
                        },
                        _ = deadline => {},
                    }
                }
            }
            Err(error) => tracing::warn!(?error, "No fue posible escuchar las decisiones de alcance"),
        }
        // Solo ante pérdida real de conexión; no es sondeo de la cola vacía.
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
