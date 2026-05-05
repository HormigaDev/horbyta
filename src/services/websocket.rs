use axum::{
    extract::ws::{Message, WebSocket},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;
use futures::SinkExt;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{PaginatorTrait, QuerySelect};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    constants::USER_AUTH_COOKIE_NAME,
    dtos::websocket::WsMessage,
    entities::{notifications, prelude::Notifications},
    error::AppError,
    middlewares::{auth::Claims, origin::is_allowed_request_origin},
    services::notifications::{NotificationEvent, NotificationService},
    state::AppState,
};

const UNREAD_NOTIFICATION_BATCH_LIMIT: u64 = 20;
const UNREAD_NOTIFICATION_SUMMARY_THRESHOLD: u64 = 20;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NotificationReadPayload {
    id: Uuid,
}

#[derive(Clone)]
pub struct WebSocketService {
    state: AppState,
}

impl WebSocketService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn authenticate_handshake(&self, headers: &HeaderMap) -> Result<(Uuid, String), StatusCode> {
        self.validate_origin(headers)?;

        let jar = CookieJar::from_headers(headers);
        let token = jar
            .get(USER_AUTH_COOKIE_NAME)
            .map(|cookie| cookie.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok((token_data.claims.sub, Uuid::now_v7().to_string()))
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<NotificationEvent> {
        self.state.notification_bus.subscribe()
    }

    pub async fn send_unread_notifications(
        &self,
        user_id: Uuid,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<(), AppError> {
        let unread_count = Notifications::find()
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .count(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        let pending_count = Notifications::find()
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .filter(notifications::Column::ReceivedAt.is_null())
            .count(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        let persisted_notifications = Notifications::find()
            .filter(notifications::Column::UserId.eq(user_id))
            .order_by_asc(notifications::Column::CreatedAt)
            .all(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        for notification in persisted_notifications.iter().filter(|notification| {
            notification.is_read || (!notification.is_read && notification.received_at.is_some())
        }) {
            self.send_silent_notification_event(sender, notification).await?;
        }

        if pending_count > UNREAD_NOTIFICATION_SUMMARY_THRESHOLD {
            self.send_notification_summary_event(sender, unread_count).await?;
            for notification in persisted_notifications
                .iter()
                .filter(|notification| !notification.is_read && notification.received_at.is_none())
            {
                self.send_silent_notification_event(sender, notification).await?;
            }
            NotificationService::mark_unread_as_received(self.state.db.as_ref(), user_id).await?;
            self.send_unread_count_event(sender, unread_count).await?;
            return Ok(());
        }

        let pending_notifications = Notifications::find()
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .filter(notifications::Column::ReceivedAt.is_null())
            .order_by_asc(notifications::Column::CreatedAt)
            .limit(UNREAD_NOTIFICATION_BATCH_LIMIT)
            .all(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        let received_ids = pending_notifications
            .iter()
            .map(|notification| notification.id)
            .collect::<Vec<_>>();

        for notification in &pending_notifications {
            self.send_notification_event(sender, &notification).await?;
        }

        NotificationService::mark_as_received(self.state.db.as_ref(), user_id, received_ids).await?;
        self.send_unread_count_event(sender, unread_count).await?;

        Ok(())
    }

    pub async fn process_message(
        &self,
        text: &str,
        _user_id: Uuid,
        _session_id: &str,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<bool, AppError> {
        let ws_msg: WsMessage =
            serde_json::from_str(text).map_err(|_| AppError::BadRequest("invalid message format".to_string()))?;

        tracing::debug!("WebSocket message received: {:?}", ws_msg);

        match ws_msg.msg_type.as_str() {
            "ping" => {
                self.send_ws_message(
                    sender,
                    &WsMessage {
                        msg_type: "pong".to_string(),
                        data: None,
                        id: ws_msg.id,
                    },
                )
                .await?;
                Ok(true)
            }
            "close" => Ok(false),
            "notification.read" => {
                let data = ws_msg
                    .data
                    .ok_or_else(|| AppError::BadRequest("missing notification read payload".to_string()))?;
                let payload: NotificationReadPayload = serde_json::from_value(data)
                    .map_err(|_| AppError::BadRequest("invalid notification read payload".to_string()))?;

                NotificationService::mark_as_read(self.state.db.as_ref(), _user_id, payload.id).await?;
                Ok(true)
            }
            "notification.read_all" => {
                NotificationService::mark_all_as_read(self.state.db.as_ref(), _user_id).await?;
                Ok(true)
            }
            "notification.clear_all" => {
                NotificationService::delete_all(self.state.db.as_ref(), _user_id).await?;
                Ok(true)
            }
            _ => Err(AppError::BadRequest("unknown message type".to_string())),
        }
    }

    pub async fn send_processing_error(&self, sender: &mut futures::stream::SplitSink<WebSocket, Message>) {
        let _ = self
            .send_ws_message(
                sender,
                &WsMessage {
                    msg_type: "error".to_string(),
                    data: Some(json!({"message": "Message processing failed"})),
                    id: None,
                },
            )
            .await;
    }

    pub async fn send_notification_event(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        notification: &notifications::Model,
    ) -> Result<(), AppError> {
        self.send_notification_event_with_options(sender, notification, false)
            .await
    }

    async fn send_silent_notification_event(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        notification: &notifications::Model,
    ) -> Result<(), AppError> {
        self.send_notification_event_with_options(sender, notification, true)
            .await
    }

    async fn send_notification_event_with_options(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        notification: &notifications::Model,
        silent: bool,
    ) -> Result<(), AppError> {
        let event = WsMessage {
            msg_type: "notification".to_string(),
            data: Some(json!({
                "id": notification.id,
                "message": notification.title,
                "title": notification.title,
                "body": notification.body,
                "type": notification.severity,
                "source": "async",
                "preferenceKey": notification.preference_key,
                "eventType": notification.event_type,
                "entityType": notification.entity_type,
                "entityId": notification.entity_id,
                "payload": notification.payload,
                "silent": silent,
                "read": notification.is_read,
                "createdAt": notification.created_at
            })),
            id: Some(notification.id.to_string()),
        };

        self.send_ws_message(sender, &event).await
    }

    pub async fn send_notification_event_and_mark_received(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        notification: &notifications::Model,
    ) -> Result<(), AppError> {
        self.send_notification_event(sender, notification).await?;
        NotificationService::mark_as_received(
            self.state.db.as_ref(),
            notification.user_id,
            vec![notification.id],
        )
        .await
    }

    async fn send_notification_summary_event(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        unread_count: u64,
    ) -> Result<(), AppError> {
        let event = WsMessage {
            msg_type: "notification".to_string(),
            data: Some(json!({
                "type": "info",
                "source": "async",
                "summaryType": "unread_notifications",
                "unreadCount": unread_count
            })),
            id: None,
        };

        self.send_ws_message(sender, &event).await
    }

    async fn send_unread_count_event(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        unread_count: u64,
    ) -> Result<(), AppError> {
        let event = WsMessage {
            msg_type: "notification.unread_count".to_string(),
            data: Some(json!({ "count": unread_count })),
            id: None,
        };

        self.send_ws_message(sender, &event).await
    }

    async fn send_ws_message(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        message: &WsMessage,
    ) -> Result<(), AppError> {
        let payload = serde_json::to_string(message)
            .map_err(|_| AppError::Internal("failed serializing websocket message".to_string()))?;

        sender
            .send(Message::Text(payload.into()))
            .await
            .map_err(|_| AppError::Internal("failed sending websocket message".to_string()))
    }

    fn validate_origin(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        if !is_allowed_request_origin(headers, &self.state.config) {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(())
    }
}
