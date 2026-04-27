use sea_orm::DatabaseConnection;
use serde_json::{Value as JsonValue, json};
use std::{collections::BTreeMap, sync::Arc};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::dtos::websocket::WsMessage;
use crate::error::AppError;
use crate::services::ai_tools;

// Include the generated code from proto/horbyta_service.proto
// This assumes tonic-build compiled the proto during cargo build
include!(concat!(env!("OUT_DIR"), "/horbyta_service.rs"));

pub struct AiService {
    client: ai_service_client::AiServiceClient<Channel>,
    db: Arc<DatabaseConnection>,
}

impl AiService {
    pub async fn connect(url: &str, db: Arc<DatabaseConnection>) -> Result<Self, AppError> {
        let channel = Channel::from_shared(url.to_string())
            .map_err(|_| AppError::Internal(format!("invalid ai service url: {}", url)))?
            .connect()
            .await
            .map_err(|e| AppError::Internal(format!("failed to connect to ai service: {}", e)))?;

        Ok(Self {
            client: ai_service_client::AiServiceClient::new(channel),
            db,
        })
    }

    pub async fn stream_chat_response(
        &mut self,
        user_id: Uuid,
        session_id: &str,
        model: &str,
        api_key: &str,
        messages: Vec<Message>,
        sender: &mut futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    ) -> Result<(), AppError> {
        let available_tools = build_tool_definitions()?;
        let request = tonic::Request::new(ChatRequest {
            messages,
            available_tools,
        });

        let mut request = request;
        request.metadata_mut().insert(
            "x-user-id",
            user_id
                .to_string()
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::Internal("invalid user id format".to_string()))?,
        );
        request.metadata_mut().insert(
            "x-session-id",
            session_id
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::Internal("invalid session id format".to_string()))?,
        );
        request.metadata_mut().insert(
            "x-model",
            model
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::BadRequest("invalid model value".to_string()))?,
        );
        request.metadata_mut().insert(
            "x-api-key",
            api_key
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::BadRequest("invalid api key value".to_string()))?,
        );

        let mut stream = self
            .client
            .chat(request)
            .await
            .map_err(|e| AppError::Internal(format!("grpc request failed: {}", e)))?
            .into_inner();

        let mut full_content = String::new();

        while let Some(event_result) = futures::stream::StreamExt::next(&mut stream).await {
            let event = event_result.map_err(|e| AppError::Internal(format!("stream error from ai service: {}", e)))?;

            match event.payload {
                Some(chat_event::Payload::TextDelta(delta)) => {
                    full_content.push_str(&delta.text);
                    send_typing_message(sender, &delta.text).await?;
                }
                Some(chat_event::Payload::ToolCall(tool_call)) => {
                    send_tool_status_message(sender, "running", &tool_call.name, &tool_call.id, None).await?;

                    let tool_args = match tool_call.arguments {
                        Some(arguments) => struct_to_json_value(&arguments),
                        None => JsonValue::Object(serde_json::Map::new()),
                    };

                    let exec_result =
                        ai_tools::execute_tool(self.db.clone(), user_id, &tool_call.name, tool_args).await;

                    let (submit_payload, status_details, status_kind) = match exec_result {
                        Ok(result) => {
                            let details = build_tool_status_details(&tool_call.name, &result);
                            (
                                ToolResultRequest {
                                    tool_call_id: tool_call.id.clone(),
                                    success: true,
                                    result: Some(json_value_to_struct(result)?),
                                    error_message: String::new(),
                                },
                                details,
                                "completed",
                            )
                        }
                        Err(err) => {
                            let error_text = err.to_string();
                            (
                                ToolResultRequest {
                                    tool_call_id: tool_call.id.clone(),
                                    success: false,
                                    result: None,
                                    error_message: error_text.clone(),
                                },
                                Some(json!({
                                    "message": format!("Error ejecutando {}", tool_call.name),
                                    "error": error_text,
                                })),
                                "error",
                            )
                        }
                    };

                    self.submit_tool_result(session_id, submit_payload).await?;

                    send_tool_status_message(sender, status_kind, &tool_call.name, &tool_call.id, status_details)
                        .await?;
                }
                Some(chat_event::Payload::Done(_)) => {
                    send_complete_message(sender, &full_content).await?;
                    break;
                }
                Some(chat_event::Payload::Error(err)) => {
                    tracing::error!("AI service error: {} - {}", err.code, err.message);
                    return Err(AppError::Internal(format!("ai service error: {}", err.message)));
                }
                None => {
                    tracing::warn!("Received event with no payload");
                }
            }
        }

        Ok(())
    }

    async fn submit_tool_result(&mut self, session_id: &str, payload: ToolResultRequest) -> Result<(), AppError> {
        let mut request = tonic::Request::new(payload);

        request.metadata_mut().insert(
            "x-session-id",
            session_id
                .parse::<MetadataValue<_>>()
                .map_err(|_| AppError::Internal("invalid session id format".to_string()))?,
        );

        self.client
            .submit_tool_result(request)
            .await
            .map_err(|e| AppError::Internal(format!("submit tool result failed: {}", e)))?;

        Ok(())
    }
}

fn build_tool_definitions() -> Result<Vec<ToolDefinition>, AppError> {
    ai_tools::tool_specs()
        .into_iter()
        .map(|tool| {
            Ok(ToolDefinition {
                name: tool.name.to_string(),
                description: tool.description.to_string(),
                parameters: Some(json_value_to_struct(tool.parameters)?),
            })
        })
        .collect()
}

fn json_value_to_struct(value: JsonValue) -> Result<prost_types::Struct, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| AppError::BadRequest("tool payload must be a json object".to_string()))?;

    let fields = object
        .iter()
        .map(|(k, v)| Ok((k.clone(), json_value_to_proto_value(v)?)))
        .collect::<Result<BTreeMap<String, prost_types::Value>, AppError>>()?;

    Ok(prost_types::Struct { fields })
}

fn json_value_to_proto_value(value: &JsonValue) -> Result<prost_types::Value, AppError> {
    let kind = match value {
        JsonValue::Null => prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue as i32),
        JsonValue::Bool(v) => prost_types::value::Kind::BoolValue(*v),
        JsonValue::Number(v) => {
            let num = v
                .as_f64()
                .ok_or_else(|| AppError::BadRequest("invalid json number".to_string()))?;
            prost_types::value::Kind::NumberValue(num)
        }
        JsonValue::String(v) => prost_types::value::Kind::StringValue(v.clone()),
        JsonValue::Array(arr) => {
            let values = arr
                .iter()
                .map(json_value_to_proto_value)
                .collect::<Result<Vec<prost_types::Value>, AppError>>()?;
            prost_types::value::Kind::ListValue(prost_types::ListValue { values })
        }
        JsonValue::Object(obj) => {
            let fields = obj
                .iter()
                .map(|(k, v)| Ok((k.clone(), json_value_to_proto_value(v)?)))
                .collect::<Result<BTreeMap<String, prost_types::Value>, AppError>>()?;

            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };

    Ok(prost_types::Value { kind: Some(kind) })
}

fn struct_to_json_value(value: &prost_types::Struct) -> JsonValue {
    let map = value
        .fields
        .iter()
        .map(|(k, v)| (k.clone(), proto_value_to_json_value(v)))
        .collect::<serde_json::Map<String, JsonValue>>();

    JsonValue::Object(map)
}

fn proto_value_to_json_value(value: &prost_types::Value) -> JsonValue {
    match value.kind.as_ref() {
        Some(prost_types::value::Kind::NullValue(_)) => JsonValue::Null,
        Some(prost_types::value::Kind::NumberValue(v)) => {
            if v.is_finite() && v.fract() == 0.0 {
                if *v >= 0.0 && *v <= u64::MAX as f64 {
                    return JsonValue::Number(serde_json::Number::from(*v as u64));
                }

                if *v >= i64::MIN as f64 && *v <= i64::MAX as f64 {
                    return JsonValue::Number(serde_json::Number::from(*v as i64));
                }
            }

            serde_json::Number::from_f64(*v).map_or(JsonValue::Null, JsonValue::Number)
        }
        Some(prost_types::value::Kind::StringValue(v)) => JsonValue::String(v.clone()),
        Some(prost_types::value::Kind::BoolValue(v)) => JsonValue::Bool(*v),
        Some(prost_types::value::Kind::StructValue(v)) => struct_to_json_value(v),
        Some(prost_types::value::Kind::ListValue(v)) => {
            JsonValue::Array(v.values.iter().map(proto_value_to_json_value).collect())
        }
        None => JsonValue::Null,
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
        .map_err(|_| AppError::Internal("failed serializing websocket message".to_string()))?;

    sender
        .send(WsMessageRaw::Text(payload.into()))
        .await
        .map_err(|_| AppError::Internal("failed sending websocket message".to_string()))
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
        .map_err(|_| AppError::Internal("failed serializing websocket message".to_string()))?;

    sender
        .send(WsMessageRaw::Text(payload.into()))
        .await
        .map_err(|_| AppError::Internal("failed sending websocket message".to_string()))
}

async fn send_tool_status_message(
    sender: &mut futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    status: &str,
    tool_name: &str,
    tool_call_id: &str,
    details: Option<JsonValue>,
) -> Result<(), AppError> {
    use axum::extract::ws::Message as WsMessageRaw;
    use futures::SinkExt;

    let tool_status_msg = WsMessage {
        msg_type: "tool_status".to_string(),
        data: Some(json!({
            "status": status,
            "tool_name": tool_name,
            "tool_call_id": tool_call_id,
            "details": details
        })),
        id: None,
    };

    let payload = serde_json::to_string(&tool_status_msg)
        .map_err(|_| AppError::Internal("failed serializing websocket message".to_string()))?;

    sender
        .send(WsMessageRaw::Text(payload.into()))
        .await
        .map_err(|_| AppError::Internal("failed sending websocket message".to_string()))
}

fn build_tool_status_details(tool_name: &str, result: &JsonValue) -> Option<JsonValue> {
    match tool_name {
        "task_create" => {
            let task = result.get("task")?;
            let task_id = task.get("id")?.as_str()?;
            let title = task.get("title")?.as_str()?;

            Some(json!({
                "message": format!("Tarea creada: {}", title),
                "badge": {
                    "kind": "task",
                    "label": title,
                    "route": format!("/tasks/{}", task_id),
                    "id": task_id
                }
            }))
        }
        "project_read" => {
            let project = result.get("project")?;
            let project_id = project.get("id")?.as_str()?;
            let name = project.get("name")?.as_str()?;

            Some(json!({
                "message": format!("Proyecto leido: {}", name),
                "badge": {
                    "kind": "project",
                    "label": name,
                    "route": format!("/projects/{}", project_id),
                    "id": project_id
                }
            }))
        }
        "task_read" => {
            let task = result.get("task")?;
            let task_id = task.get("id")?.as_str()?;
            let title = task.get("title")?.as_str()?;

            Some(json!({
                "message": format!("Tarea leida: {}", title),
                "badge": {
                    "kind": "task",
                    "label": title,
                    "route": format!("/tasks/{}", task_id),
                    "id": task_id
                }
            }))
        }
        "projects_list" => {
            let total = result
                .get("total")
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    result
                        .get("items")
                        .and_then(|v| v.as_array().map(|arr| arr.len() as u64))
                })
                .unwrap_or(0);

            Some(json!({
                "message": format!("Proyectos leidos: {}", total),
                "badge": {
                    "kind": "projects",
                    "label": format!("{} proyectos", total),
                    "count": total
                }
            }))
        }
        "tasks_list" => {
            let total = result
                .get("total")
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    result
                        .get("items")
                        .and_then(|v| v.as_array().map(|arr| arr.len() as u64))
                })
                .unwrap_or(0);

            Some(json!({
                "message": format!("Tareas leidas: {}", total),
                "badge": {
                    "kind": "tasks",
                    "label": format!("{} tareas", total),
                    "count": total
                }
            }))
        }
        _ => None,
    }
}
