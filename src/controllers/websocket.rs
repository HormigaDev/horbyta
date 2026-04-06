use axum::{
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::StreamExt;
use uuid::Uuid;

use crate::{services::websocket::WebSocketService, state::AppState};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let service = WebSocketService::new(state.clone());
    let (user_id, session_id) = service.authenticate_handshake(&headers)?;

    Ok(ws.on_upgrade(move |socket| {
        let service = WebSocketService::new(state);
        handle_socket(socket, user_id, session_id, service)
    }))
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, session_id: String, service: WebSocketService) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        tracing::info!(
            user_id = %user_id,
            session_id = %session_id,
            "WebSocket connection established"
        );

        if let Err(error) = service.send_unread_notifications(user_id, &mut sender).await {
            tracing::error!("Failed to send unread notifications: {}", error);
        }

        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                match service.process_message(&text, user_id, &session_id, &mut sender).await {
                    Ok(should_continue) if !should_continue => break,
                    Err(error) => {
                        tracing::error!("WebSocket message processing error: {}", error);
                        service.send_processing_error(&mut sender).await;
                        break;
                    }
                    _ => {}
                }
            }
        }

        tracing::info!(
            user_id = %user_id,
            session_id = %session_id,
            "WebSocket connection closed"
        );
    });
}
