use axum::{
    extract::ws::{Message, WebSocket},
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;
use futures::SinkExt;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    dtos::{user::UserPreferences, websocket::WsMessage},
    entities::{conversation_history::Entity as ConversationHistory, notifications, prelude::Users, prelude::*},
    error::AppError,
    middlewares::auth::Claims,
    services::ai::Message as ProtoMessage,
    state::AppState,
};

const CHATBOT_NOTIFICATION_KEY: &str = "clients_projects.comment_or_review";

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
        user_id: Uuid,
        session_id: &str,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<bool, AppError> {
        let ws_msg: WsMessage =
            serde_json::from_str(text).map_err(|_| AppError::BadRequest("Invalid message format".to_string()))?;

        tracing::debug!("WebSocket message received: {:?}", ws_msg);

        match ws_msg.msg_type.as_str() {
            "chat" => {
                self.handle_chat_message(&ws_msg, user_id, session_id, sender).await?;
                Ok(true)
            }
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
            _ => Err(AppError::BadRequest("Unknown message type".to_string())),
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

    async fn handle_chat_message(
        &self,
        msg: &WsMessage,
        user_id: Uuid,
        session_id: &str,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<(), AppError> {
        let data = msg
            .data
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Missing data field".to_string()))?;

        let chat_id = data
            .get("chat_id")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::BadRequest("Missing chat_id in data".to_string()))?
            .parse::<Uuid>()
            .map_err(|_| AppError::BadRequest("Invalid chat_id format".to_string()))?;

        let user_content = data
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| AppError::BadRequest("Missing content in data".to_string()))?;

        tracing::info!(
            user_id = %user_id,
            chat_id = %chat_id,
            message = %user_content,
            "Processing chat message"
        );

        self.send_generating_status(sender).await?;

        let chat = ConversationHistory::find_by_id(chat_id)
            .one(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("Chat not found".to_string()))?;

        let messages = self.build_proto_messages(&chat.messages, user_content)?;

        let mut ai_service = self.state.ai_service.lock().await;
        ai_service
            .stream_chat_response(user_id, session_id, messages, sender)
            .await?;

        if let Some(notification) = self.create_chat_notification(user_id, session_id, data).await? {
            self.send_notification_event(sender, &notification).await?
        }

        Ok(())
    }

    fn build_proto_messages(&self, messages_json: &Value, user_content: &str) -> Result<Vec<ProtoMessage>, AppError> {
        let mut messages = Vec::new();

        if let Some(arr) = messages_json.as_array() {
            for msg in arr {
                let role = msg.get("role").and_then(Value::as_str).unwrap_or("user");
                let content = msg.get("content").and_then(Value::as_str).unwrap_or("");

                let role_enum = match role {
                    "bot" | "assistant" => 2,
                    "system" => 3,
                    _ => 1,
                };

                messages.push(ProtoMessage {
                    role: role_enum,
                    content: content.to_string(),
                    tool_call: None,
                    tool_result: None,
                });
            }
        }

        messages.push(ProtoMessage {
            role: 1,
            content: user_content.to_string(),
            tool_call: None,
            tool_result: None,
        });

        Ok(messages)
    }

    async fn send_generating_status(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<(), AppError> {
        let status_msg = WsMessage {
            msg_type: "generating".to_string(),
            data: None,
            id: None,
        };
        self.send_ws_message(sender, &status_msg).await
    }

    async fn create_chat_notification(
        &self,
        user_id: Uuid,
        session_id: &str,
        message: &Value,
    ) -> Result<Option<notifications::Model>, AppError> {
        let user = Users::find_by_id(user_id)
            .one(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Unauthorized("Authenticated user not found".to_string()))?;

        let preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        if !is_notification_enabled(CHATBOT_NOTIFICATION_KEY, &preferences) {
            return Ok(None);
        }

        let message_preview = message
            .as_object()
            .and_then(|value| value.get("content"))
            .and_then(Value::as_str)
            .map(|content| content.chars().take(140).collect::<String>());

        let body = message_preview.map(|preview| format!("IA respondió al chat {}: {}", session_id, preview));

        let notification = notifications::ActiveModel {
            user_id: Set(user_id),
            preference_key: Set(CHATBOT_NOTIFICATION_KEY.to_string()),
            title: Set("Nueva respuesta del asistente".to_string()),
            body: Set(body),
            payload: Set(Some(json!({
                "session_id": session_id,
                "source": "chatbot",
                "kind": "assistant_reply"
            }))),
            severity: Set("info".to_string()),
            source: Set("chatbot".to_string()),
            is_read: Set(false),
            read_at: Set(None),
            ..Default::default()
        }
        .insert(self.state.db.as_ref())
        .await
        .map_err(AppError::Database)?;

        Ok(Some(notification))
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
            .map_err(|_| AppError::Internal("Failed serializing websocket message".to_string()))?;

        sender
            .send(Message::Text(payload.into()))
            .await
            .map_err(|_| AppError::Internal("Failed sending websocket message".to_string()))
    }

    fn validate_origin(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        if let (Some(origin), Some(host)) = (headers.get("origin"), headers.get("host")) {
            let origin = origin.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            let host = host.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            let same_origin = origin.contains(&format!("//{host}"));

            if !same_origin {
                return Err(StatusCode::FORBIDDEN);
            }
        }

        Ok(())
    }
}

fn is_notification_enabled(preference_key: &str, preferences: &UserPreferences) -> bool {
    match preference_key {
        "clients_projects.proposal_decision" => preferences.notifications.clients_projects.proposal_decision,
        "clients_projects.deliverable_approved" => preferences.notifications.clients_projects.deliverable_approved,
        "clients_projects.comment_or_review" => preferences.notifications.clients_projects.comment_or_review,
        "clients_projects.added_to_new_project" => preferences.notifications.clients_projects.added_to_new_project,
        "deliveries_deadlines.deadline_24h_48h" => preferences.notifications.deliveries_deadlines.deadline_24h_48h,
        "deliveries_deadlines.deliverable_overdue" => {
            preferences.notifications.deliveries_deadlines.deliverable_overdue
        }
        "deliveries_deadlines.milestone_completed" => {
            preferences.notifications.deliveries_deadlines.milestone_completed
        }
        "contracts.contract_sent" => preferences.notifications.contracts.contract_sent,
        "contracts.contract_signed" => preferences.notifications.contracts.contract_signed,
        "contracts.contract_expiring" => preferences.notifications.contracts.contract_expiring,
        "billing.invoice_sent" => preferences.notifications.billing.invoice_sent,
        "billing.invoice_overdue_unpaid" => preferences.notifications.billing.invoice_overdue_unpaid,
        "billing.payment_received" => preferences.notifications.billing.payment_received,
        "billing.auto_payment_reminder" => preferences.notifications.billing.auto_payment_reminder,
        "time_tracking.long_running_timer" => preferences.notifications.time_tracking.long_running_timer,
        "time_tracking.weekly_hours_summary" => preferences.notifications.time_tracking.weekly_hours_summary,
        _ => false,
    }
}
