use axum::{
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use futures::{SinkExt, StreamExt};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{middlewares::auth::Claims, state::AppState};

#[derive(Serialize, Deserialize, Debug)]
pub struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    if let (Some(origin), Some(host)) = (headers.get("origin"), headers.get("host")) {
        let origin = origin.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        let host = host.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        let same_origin = origin.contains(&format!("//{host}"));

        if !same_origin {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let jar = CookieJar::from_headers(&headers);
    let token = jar
        .get("hormiga_auth")
        .map(|cookie| cookie.value())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validar y extraer claims del JWT
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_id = token_data.claims.sub;
    let session_id = Uuid::now_v7().to_string();

    // Aceptar la actualización de WebSocket
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, user_id, session_id)))
}

async fn handle_socket(socket: WebSocket, user_id: Uuid, session_id: String) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        // Conectar al servicio gRPC Python
        // Por ahora solo registrar la conexión (implementar después con stubs gRPC)
        tracing::info!(
            user_id = %user_id,
            session_id = %session_id,
            "WebSocket connection established"
        );

        while let Some(Ok(msg)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                match process_ws_message(&text, user_id, &session_id, &mut sender).await {
                    Ok(should_continue) if !should_continue => break,
                    Err(e) => {
                        tracing::error!("WebSocket message processing error: {}", e);
                        let _ = sender
                            .send(axum::extract::ws::Message::Text(
                                serde_json::to_string(&WsMessage {
                                    msg_type: "error".to_string(),
                                    data: Some(json!({"message": "Message processing failed"})),
                                    id: None,
                                })
                                .unwrap_or_else(|_| "{}".to_string())
                                .into(),
                            ))
                            .await;
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

async fn process_ws_message(
    text: &str,
    user_id: Uuid,
    session_id: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
) -> Result<bool, String> {
    let ws_msg: WsMessage = serde_json::from_str(text).map_err(|e| format!("Invalid message format: {}", e))?;

    tracing::debug!("WebSocket message received: {:?}", ws_msg);

    match ws_msg.msg_type.as_str() {
        "chat" => {
            handle_chat_message(&ws_msg, user_id, session_id, sender).await?;
            Ok(true)
        }
        "ping" => {
            let response = WsMessage {
                msg_type: "pong".to_string(),
                data: None,
                id: ws_msg.id,
            };
            let _ = sender
                .send(axum::extract::ws::Message::Text(
                    serde_json::to_string(&response)
                        .unwrap_or_else(|_| "{}".to_string())
                        .into(),
                ))
                .await;
            Ok(true)
        }
        "close" => Ok(false),
        _ => Err(format!("Unknown message type: {}", ws_msg.msg_type)),
    }
}

async fn handle_chat_message(
    msg: &WsMessage,
    user_id: Uuid,
    session_id: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
) -> Result<(), String> {
    let data = msg.data.as_ref().ok_or_else(|| "Missing data field".to_string())?;

    tracing::info!(
        user_id = %user_id,
        session_id = %session_id,
        message = ?data,
        "Processing chat message"
    );

    // TODO: Conectar con el servicio gRPC Python
    // Por ahora enviar un mensaje dummy de confirmación
    let response = WsMessage {
        msg_type: "message".to_string(),
        data: Some(json!({
            "content": "Echo: ".to_string() + &data.to_string(),
            "role": "assistant"
        })),
        id: msg.id.clone(),
    };

    let _ = sender
        .send(axum::extract::ws::Message::Text(
            serde_json::to_string(&response)
                .unwrap_or_else(|_| "{}".to_string())
                .into(),
        ))
        .await;

    Ok(())
}
