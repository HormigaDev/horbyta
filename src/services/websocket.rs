use axum::{
    extract::ws::{Message, WebSocket},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;
use futures::SinkExt;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::QuerySelect;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::websocket::WsMessage,
    entities::{notifications, prelude::Notifications},
    error::AppError,
    middlewares::auth::Claims,
    state::AppState,
};

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
            .get("hormiga_auth")
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

    pub async fn send_unread_notifications(
        &self,
        user_id: Uuid,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<(), AppError> {
        let unread_notifications = Notifications::find()
            .filter(notifications::Column::UserId.eq(user_id))
            .filter(notifications::Column::IsRead.eq(false))
            .order_by_asc(notifications::Column::CreatedAt)
            .limit(20)
            .all(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        for notification in unread_notifications {
            self.send_notification_event(sender, &notification).await?;
        }

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

    async fn send_notification_event(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        notification: &notifications::Model,
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
                "payload": notification.payload,
                "createdAt": notification.created_at
            })),
            id: Some(notification.id.to_string()),
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
        if let (Some(origin), Some(host)) = (headers.get("origin"), headers.get("host")) {
            let origin = origin.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            let host = host.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            let same_origin = origin.contains(&format!("//{host}"));
            let allowed_dev_origin = !self.state.config.is_production && origin == self.state.config.frontend_origin;

            if !same_origin && !allowed_dev_origin {
                return Err(StatusCode::FORBIDDEN);
            }
        }

        Ok(())
    }
}
