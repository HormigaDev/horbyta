use serde_json::json;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::dtos::websocket::WsMessage;
use crate::error::AppError;

// Include the generated code from proto/horbyta_service.proto
// This assumes tonic-build compiled the proto during cargo build
include!(concat!(env!("OUT_DIR"), "/horbyta_service.rs"));

pub struct AiService {
    client: ai_service_client::AiServiceClient<Channel>,
}

impl AiService {
    pub async fn connect(url: &str) -> Result<Self, AppError> {
        let channel = Channel::from_shared(url.to_string())
            .map_err(|_| AppError::Internal(format!("Invalid AI service URL: {}", url)))?
            .connect()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to AI service: {}", e)))?;

        Ok(Self {
            client: ai_service_client::AiServiceClient::new(channel),
        })
    }

    pub async fn stream_chat_response(
        &mut self,
        user_id: Uuid,
        session_id: &str,
        messages: Vec<Message>,
        sender: &mut futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    ) -> Result<(), AppError> {
        let request = tonic::Request::new(ChatRequest {
            messages,
            available_tools: vec![],
        });

        let mut request = request;
        request.metadata_mut().insert(
            "x-user-id",
            user_id
                .to_string()
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::Internal("Invalid user ID format".to_string()))?,
        );
        request.metadata_mut().insert(
            "x-session-id",
            session_id
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::Internal("Invalid session ID format".to_string()))?,
        );

        let mut stream = self
            .client
            .chat(request)
            .await
            .map_err(|e| AppError::Internal(format!("gRPC request failed: {}", e)))?
            .into_inner();

        let mut full_content = String::new();

        while let Some(event_result) = futures::stream::StreamExt::next(&mut stream).await {
            let event = event_result.map_err(|e| AppError::Internal(format!("Stream error from AI service: {}", e)))?;

            match event.payload {
                Some(chat_event::Payload::TextDelta(delta)) => {
                    full_content.push_str(&delta.text);
                    send_typing_message(sender, &delta.text).await?;
                }
                Some(chat_event::Payload::ToolCall(_)) => {
                    tracing::warn!("Tool calls not yet supported");
                }
                Some(chat_event::Payload::Done(_)) => {
                    send_complete_message(sender, &full_content).await?;
                    break;
                }
                Some(chat_event::Payload::Error(err)) => {
                    tracing::error!("AI service error: {} - {}", err.code, err.message);
                    return Err(AppError::Internal(format!("AI service error: {}", err.message)));
                }
                None => {
                    tracing::warn!("Received event with no payload");
                }
            }
        }

        Ok(())
    }
}

async fn send_typing_message(
    sender: &mut futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    chunk: &str,
) -> Result<(), AppError> {
    use axum::extract::ws::Message as WsMessageRaw;
    use futures::SinkExt;

    let typing_msg = WsMessage {
        msg_type: "typing".to_string(),
        data: Some(json!({
            "delta": chunk,
            "role": "model"
        })),
        id: None,
    };

    let payload = serde_json::to_string(&typing_msg)
        .map_err(|_| AppError::Internal("Failed serializing websocket message".to_string()))?;

    sender
        .send(WsMessageRaw::Text(payload.into()))
        .await
        .map_err(|_| AppError::Internal("Failed sending websocket message".to_string()))
}

async fn send_complete_message(
    sender: &mut futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    content: &str,
) -> Result<(), AppError> {
    use axum::extract::ws::Message as WsMessageRaw;
    use futures::SinkExt;

    let complete_msg = WsMessage {
        msg_type: "message".to_string(),
        data: Some(json!({
            "content": content,
            "role": "model"
        })),
        id: None,
    };

    let payload = serde_json::to_string(&complete_msg)
        .map_err(|_| AppError::Internal("Failed serializing websocket message".to_string()))?;

    sender
        .send(WsMessageRaw::Text(payload.into()))
        .await
        .map_err(|_| AppError::Internal("Failed sending websocket message".to_string()))
}
