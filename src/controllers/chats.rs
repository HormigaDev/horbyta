use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        chat::{CreateChatDto, UpdateChatMessagesDto, UpdateChatTitleDto, chat_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::chats::ChatService,
    state::AppState,
};

async fn search_chats(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    chat_filter_schema().validate_payload(&payload)?;

    let (items, total) = ChatService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total
    })))
}

async fn get_chat(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let chat = ChatService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(chat))
}

async fn create_chat(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateChatDto>,
) -> Result<impl IntoResponse, AppError> {
    let chat = ChatService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(chat))
}

async fn update_chat_title(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateChatTitleDto>,
) -> Result<impl IntoResponse, AppError> {
    let chat = ChatService::update_title(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(chat))
}

async fn update_chat_messages(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateChatMessagesDto>,
) -> Result<impl IntoResponse, AppError> {
    let chat = ChatService::update_messages(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(chat))
}

async fn delete_chat(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ChatService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Chat eliminado" })))
}

pub fn chats_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_chats))
        .route("/", post(create_chat))
        .route("/{id}", get(get_chat).delete(delete_chat))
        .route("/{id}/title", axum::routing::patch(update_chat_title))
        .route("/{id}/messages", axum::routing::patch(update_chat_messages))
}
