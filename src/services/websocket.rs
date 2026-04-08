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
    dtos::{
        user::UserPreferences,
        websocket::{ChatClientPayload, WsMessage},
    },
    entities::{conversation_history::Entity as ConversationHistory, notifications, prelude::Users, prelude::*},
    error::AppError,
    middlewares::auth::Claims,
    services::ai::{AiService, Message as ProtoMessage},
    state::AppState,
    utils::crypto::decrypt_secret,
};

const CHATBOT_NOTIFICATION_KEY: &str = "clients_projects.comment_or_review";
const DEFAULT_CHAT_TITLE: &str = "Nuevo chat";

struct AiRuntimeConfig {
    model: String,
    api_key: String,
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
        _session_id: &str,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    ) -> Result<(), AppError> {
        let data = msg
            .data
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Missing data field".to_string()))?;

        let payload: ChatClientPayload = serde_json::from_value(data.clone())
            .map_err(|_| AppError::BadRequest("Invalid chat payload".to_string()))?;

        let requested_chat_id = payload
            .chat_id
            .parse::<Uuid>()
            .map_err(|_| AppError::BadRequest("Invalid chat_id format".to_string()))?;

        let user_content = payload.content.as_str();

        tracing::info!(
            user_id = %user_id,
            chat_id = %requested_chat_id,
            message = %user_content,
            "Processing chat message"
        );

        self.send_generating_status(sender).await?;
        self.send_ai_runtime_status(sender, "processing", "Analizando solicitud", None)
            .await?;

        let (chat, effective_chat_id, created_new_chat) = self
            .resolve_or_create_chat(user_id, requested_chat_id, user_content)
            .await?;

        if created_new_chat {
            self.send_chat_context(sender, requested_chat_id, effective_chat_id, &chat.title)
                .await?;
        }

        let include_user_content = !created_new_chat;
        let messages = self.build_proto_messages(&chat.messages, user_content, include_user_content)?;
        let ai_runtime = match self.resolve_ai_runtime_config(user_id).await {
            Ok(value) => value,
            Err(err) => {
                let details = Self::build_ai_error_details(&err);
                let _ = self
                    .send_ai_runtime_status(sender, "error", details.message.as_str(), Some(details.payload))
                    .await;
                return Ok(());
            }
        };

        let chat_id_for_notification = effective_chat_id.to_string();
        let ai_session_id = chat.session_id.to_string();

        let mut ai_service_guard = self.state.ai_service.lock().await;

        if ai_service_guard.is_none() {
            match AiService::connect(&self.state.config.ai_service_url, self.state.db.clone()).await {
                Ok(service) => {
                    tracing::info!("ai_service_connected_on_demand");
                    *ai_service_guard = Some(service);
                }
                Err(error) => {
                    let unavailable = AppError::Unavailable(format!("Servicio de IA no disponible: {}", error));
                    let details = Self::build_ai_error_details(&unavailable);
                    let _ = self
                        .send_ai_runtime_status(sender, "error", details.message.as_str(), Some(details.payload))
                        .await;
                    tracing::warn!("ai_service_unavailable_on_demand: {}", error);
                    return Ok(());
                }
            }
        }

        let ai_result = match ai_service_guard.as_mut() {
            Some(ai_service) => {
                ai_service
                    .stream_chat_response(
                        user_id,
                        ai_session_id.as_str(),
                        &ai_runtime.model,
                        &ai_runtime.api_key,
                        messages,
                        sender,
                    )
                    .await
            }
            None => Err(AppError::Unavailable(
                "Servicio de IA no disponible en este momento".to_string(),
            )),
        };

        if let Err(error) = &ai_result {
            if Self::is_ai_service_unavailable_error(error) {
                *ai_service_guard = None;
            }
        }

        drop(ai_service_guard);

        match ai_result {
            Ok(()) => {
                self.send_ai_runtime_status(sender, "idle", "Proceso completado", None)
                    .await?;
            }
            Err(err) => {
                let details = Self::build_ai_error_details(&err);
                let _ = self
                    .send_ai_runtime_status(sender, "error", details.message.as_str(), Some(details.payload))
                    .await;
                tracing::error!("chat_ai_execution_error: {}", err);
                return Ok(());
            }
        }

        if let Some(notification) = self
            .create_chat_notification(user_id, &chat_id_for_notification, &payload)
            .await?
        {
            self.send_notification_event(sender, &notification).await?
        }

        Ok(())
    }

    async fn resolve_or_create_chat(
        &self,
        user_id: Uuid,
        requested_chat_id: Uuid,
        user_content: &str,
    ) -> Result<(crate::entities::conversation_history::Model, Uuid, bool), AppError> {
        let existing = ConversationHistory::find_by_id(requested_chat_id)
            .one(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        if let Some(chat) = existing {
            if chat.user_id == user_id {
                return Ok((chat, requested_chat_id, false));
            }
        }

        let user_message = json!({
            "id": Uuid::now_v7().to_string(),
            "role": "user",
            "content": user_content,
            "createdAt": chrono::Utc::now().to_rfc3339(),
        });

        let title = self.derive_chat_title(user_content);

        let created = crate::entities::conversation_history::ActiveModel {
            user_id: Set(user_id),
            session_id: Set(Uuid::now_v7()),
            title: Set(title),
            messages: Set(json!([user_message])),
            ..Default::default()
        }
        .insert(self.state.db.as_ref())
        .await
        .map_err(AppError::Database)?;

        Ok((created.clone(), created.id, true))
    }

    fn derive_chat_title(&self, content: &str) -> String {
        let title = content
            .split_whitespace()
            .take(5)
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string();

        if title.is_empty() {
            DEFAULT_CHAT_TITLE.to_string()
        } else {
            title.chars().take(64).collect::<String>()
        }
    }

    async fn send_chat_context(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        requested_chat_id: Uuid,
        effective_chat_id: Uuid,
        title: &str,
    ) -> Result<(), AppError> {
        let context_msg = WsMessage {
            msg_type: "chat_context".to_string(),
            data: Some(json!({
                "requested_chat_id": requested_chat_id,
                "effective_chat_id": effective_chat_id,
                "title": title,
                "created": true
            })),
            id: None,
        };

        self.send_ws_message(sender, &context_msg).await
    }

    fn build_proto_messages(
        &self,
        messages_json: &Value,
        user_content: &str,
        include_user_content: bool,
    ) -> Result<Vec<ProtoMessage>, AppError> {
        let mut messages = Vec::new();

        if let Some(arr) = messages_json.as_array() {
            for msg in arr {
                let role = msg.get("role").and_then(Value::as_str).unwrap_or("user");
                let content = msg.get("content").and_then(Value::as_str).unwrap_or("");

                let role_enum = match role {
                    "model" => 2,
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

        if include_user_content {
            messages.push(ProtoMessage {
                role: 1,
                content: user_content.to_string(),
                tool_call: None,
                tool_result: None,
            });
        }

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

    async fn send_ai_runtime_status(
        &self,
        sender: &mut futures::stream::SplitSink<WebSocket, Message>,
        status: &str,
        message: &str,
        details: Option<Value>,
    ) -> Result<(), AppError> {
        let status_msg = WsMessage {
            msg_type: "ai_status".to_string(),
            data: Some(json!({
                "status": status,
                "message": message,
                "details": details
            })),
            id: None,
        };

        self.send_ws_message(sender, &status_msg).await
    }

    async fn create_chat_notification(
        &self,
        user_id: Uuid,
        session_id: &str,
        message: &ChatClientPayload,
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
            .content
            .split_whitespace()
            .take(140)
            .collect::<Vec<&str>>()
            .join(" ")
            .chars()
            .take(140)
            .collect::<String>();

        let body = if message_preview.is_empty() {
            None
        } else {
            Some(format!("IA respondió al chat {}: {}", session_id, message_preview))
        };

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

    async fn resolve_ai_runtime_config(&self, user_id: Uuid) -> Result<AiRuntimeConfig, AppError> {
        let user = Users::find_by_id(user_id)
            .one(self.state.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Unauthorized("Authenticated user not found".to_string()))?;

        let preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        let encrypted_key = preferences
            .ai
            .api_key_encrypted
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Configura tu API key AI en preferencias".to_string()))?;

        let api_key = decrypt_secret(&self.state.config.ai_prefs_encryption_key, encrypted_key)?;

        let model = preferences.ai.model.trim().to_string();
        if model.is_empty() {
            return Err(AppError::BadRequest(
                "Configura un modelo AI en preferencias".to_string(),
            ));
        }

        let allowed_models = [
            "google/gemini-2.5-flash",
            "google/gemini-2.5-pro",
            "google/gemini-3-flash-preview",
            "google/gemini-3.1-pro-preview",
            "anthropic/claude-3-haiku",
            "anthropic/claude-haiku-4.5",
            "anthropic/claude-sonnet-4.5",
            "anthropic/claude-sonnet-4.6",
            "anthropic/claude-opus-4.5",
            "anthropic/claude-opus-4.6",
            "openai/gpt-5-codex",
        ];

        if !allowed_models.contains(&model.as_str()) {
            return Err(AppError::BadRequest(
                "El modelo configurado no esta permitido".to_string(),
            ));
        }

        Ok(AiRuntimeConfig { model, api_key })
    }

    fn build_ai_error_details(err: &AppError) -> AiErrorDetails {
        let raw = err.to_string();
        let lowered = raw.to_lowercase();

        if Self::is_ai_error_message_unavailable(&lowered) {
            return AiErrorDetails {
                message: "Servicio de IA no disponible en este momento. Intenta nuevamente en unos segundos"
                    .to_string(),
                payload: json!({
                    "code": "AI_UNAVAILABLE",
                    "error": raw,
                }),
            };
        }

        if lowered.contains("api key") || lowered.contains("auth") {
            return AiErrorDetails {
                message: "API key de OpenRouter invalida o no configurada".to_string(),
                payload: json!({
                    "code": "AI_AUTH",
                    "error": raw,
                }),
            };
        }

        if lowered.contains("rate") || lowered.contains("429") {
            return AiErrorDetails {
                message: "OpenRouter aplico limite de tasa. Intenta nuevamente en unos segundos".to_string(),
                payload: json!({
                    "code": "AI_RATE_LIMIT",
                    "error": raw,
                }),
            };
        }

        if lowered.contains("model") {
            return AiErrorDetails {
                message: "El modelo configurado no esta disponible".to_string(),
                payload: json!({
                    "code": "AI_MODEL",
                    "error": raw,
                }),
            };
        }

        if lowered.contains("timeout") {
            return AiErrorDetails {
                message: "El proveedor de IA tardo demasiado en responder".to_string(),
                payload: json!({
                    "code": "AI_TIMEOUT",
                    "error": raw,
                }),
            };
        }

        AiErrorDetails {
            message: "Ocurrio un error procesando la respuesta del modelo".to_string(),
            payload: json!({
                "code": "AI_UNKNOWN",
                "error": raw,
            }),
        }
    }

    fn is_ai_service_unavailable_error(err: &AppError) -> bool {
        let lowered = err.to_string().to_lowercase();
        Self::is_ai_error_message_unavailable(&lowered)
    }

    fn is_ai_error_message_unavailable(lowered: &str) -> bool {
        lowered.contains("servicio de ia no disponible")
            || lowered.contains("failed to connect to ai service")
            || lowered.contains("connection refused")
            || lowered.contains("transport error")
            || lowered.contains("tcp connect error")
            || lowered.contains("dns error")
            || lowered.contains("channel closed")
            || lowered.contains("unavailable")
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
            let allowed_dev_origin = !self.state.config.is_production && origin == self.state.config.frontend_origin;

            if !same_origin && !allowed_dev_origin {
                return Err(StatusCode::FORBIDDEN);
            }
        }

        Ok(())
    }
}

struct AiErrorDetails {
    message: String,
    payload: Value,
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
