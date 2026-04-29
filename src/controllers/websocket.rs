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
        let mut notification_rx = service.subscribe_notifications();

        tracing::info!(
            user_id = %user_id,
            session_id = %session_id,
            "WebSocket connection established"
        );

        if let Err(error) = service.send_unread_notifications(user_id, &mut sender).await {
            tracing::error!("Failed to send unread notifications: {}", error);
        }

        loop {
            tokio::select! {
                socket_msg = receiver.next() => {
                    match socket_msg {
                        Some(Ok(axum::extract::ws::Message::Text(text))) => {
                            match service.process_message(&text, user_id, &session_id, &mut sender).await {
                                Ok(should_continue) if !should_continue => break,
                                Err(error) => {
                                    tracing::error!("WebSocket message processing error: {}", error);
                                    service.send_processing_error(&mut sender).await;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        Some(Ok(axum::extract::ws::Message::Close(_))) | None => break,
                        Some(Ok(_)) => {}
                        Some(Err(error)) => {
                            tracing::error!("WebSocket receive error: {}", error);
                            break;
                        }
                    }
                }
                bus_event = notification_rx.recv() => {
                    match bus_event {
                        Ok(event) if event.user_id == user_id => {
                            if let Err(error) = service.send_notification_event(&mut sender, &event.notification).await {
                                tracing::error!("Failed to send notification event: {}", error);
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "WebSocket notification receiver lagged");
                            if let Err(error) = service.send_unread_notifications(user_id, &mut sender).await {
                                tracing::error!("Failed to resync unread notifications: {}", error);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
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
