use crate::{dtos::client::ClientAuthDto, middlewares::auth::Claims, state::AppState};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use dashmap::DashMap;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use uuid::Uuid;

struct ConnectionGuard {
    connections: Arc<DashMap<Uuid, AtomicUsize>>,
    id: Uuid,
}
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(count) = self.connections.get(&self.id) {
            count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

pub async fn client(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(s): State<AppState>,
    u: ClientAuthDto,
) -> Result<Response, StatusCode> {
    upgrade(ws, headers, s, u.user_id, Some(u.client_id), u.id)
}
fn upgrade(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
) -> Result<Response, StatusCode> {
    if !crate::middlewares::origin::is_allowed_request_origin(&headers, &s.config) {
        return Err(StatusCode::FORBIDDEN);
    }
    let (cookie, secret) = if client.is_some() {
        (crate::constants::CLIENT_AUTH_COOKIE_NAME, &s.config.jwt_secret_client)
    } else {
        (crate::constants::USER_AUTH_COOKIE_NAME, &s.config.jwt_secret)
    };
    let jar = CookieJar::from_headers(&headers);
    let token = jar.get(cookie).ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = decode::<Claims>(
        token.value(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;
    let now = chrono::Utc::now().timestamp().max(0) as usize;
    let lifetime = claims.exp.saturating_sub(now);
    if lifetime == 0 {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let count = s.ws_connections.entry(actor).or_insert_with(|| AtomicUsize::new(0));
    if count.fetch_add(1, Ordering::Relaxed) >= crate::middlewares::rate_limit::WS_MAX_CONNECTIONS {
        count.fetch_sub(1, Ordering::Relaxed);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    drop(count);
    let guard = ConnectionGuard {
        connections: s.ws_connections.clone(),
        id: actor,
    };
    let rx = s.conversation_bus.subscribe();
    Ok(ws
        .max_message_size(4096)
        .max_frame_size(4096)
        .on_upgrade(move |socket| run(socket, s, owner, client, actor, lifetime, rx, guard))
        .into_response())
}
pub(crate) async fn allowed(
    s: &AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Option<Uuid>,
) -> bool {
    s.db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT 1 AS allowed FROM users u WHERE u.id=$1 AND u.status='active' AND ($2::uuid IS NULL OR EXISTS(SELECT 1 FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=$3 AND a.user_id=u.id AND c.id=$2 AND c.status='active')) AND ($4::uuid IS NULL OR EXISTS(SELECT 1 FROM projects p WHERE p.id=$4 AND p.user_id=u.id AND ($2::uuid IS NULL OR p.client_id=$2)))",
        vec![owner.into(),client.into(),actor.into(),project.into()])).await.is_ok_and(|row| row.is_some())
}
async fn run(
    mut socket: WebSocket,
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    lifetime: usize,
    mut rx: tokio::sync::broadcast::Receiver<crate::dtos::scope::ConversationEvent>,
    _guard: ConnectionGuard,
) {
    let expires = tokio::time::sleep(std::time::Duration::from_secs(lifetime as u64));
    tokio::pin!(expires);
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        tokio::select! {
            _ = &mut expires => break,
            _ = heartbeat.tick() => {
                if !allowed(&s, owner, client, actor, None).await || socket.send(Message::Ping(Vec::new().into())).await.is_err() { break; }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Ping(data))) => { if socket.send(Message::Pong(data)).await.is_err() { break; } }
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Text(_))) | Some(Ok(Message::Binary(_))) => break,
                _ => break,
            },
            event = rx.recv() => match event {
                Ok(event) if event.user_id==owner && (client.is_none() || client==event.client_id) => {
                    if event.data["kind"] == "read" && event.data["actor_id"].as_str() != Some(actor.to_string().as_str()) { continue; }
                    if !allowed(&s, owner, client, actor, Some(event.project_id)).await { continue; }
                    let payload = json!({"type":"scope.conversation.updated","data":event.data});
                    if socket.send(Message::Text(payload.to_string().into())).await.is_err() { break; }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    if socket.send(Message::Text(json!({"type":"scope.conversation.resync"}).to_string().into())).await.is_err() { break; }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    }
    let _ = socket.send(Message::Close(None)).await;
}
