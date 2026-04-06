use axum::{
    Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        chat::{CreateChatDto, UpdateChatTitleDto, chat_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::chats::ChatService,
    state::AppState,
};

async fn search_chats(
    user: UserDto,
    service: ChatService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    chat_filter_schema().validate_payload(&payload)?;

    let (items, total) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total
    })))
}

async fn get_chat(
    user: UserDto,
    service: ChatService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let chat = service.get_by_id(user.id, id).await?;
    Ok(Json(chat))
}

async fn create_chat(
    user: UserDto,
    service: ChatService,
    ValidatedJson(payload): ValidatedJson<CreateChatDto>,
) -> Result<impl IntoResponse, AppError> {
    let chat = service.create(user.id, payload).await?;
    Ok(Json(chat))
}

async fn update_chat_title(
    user: UserDto,
    service: ChatService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateChatTitleDto>,
) -> Result<impl IntoResponse, AppError> {
    let chat = service.update_title(user.id, id, payload).await?;
    Ok(Json(chat))
}

async fn delete_chat(
    user: UserDto,
    service: ChatService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({ "message": "Chat eliminado" })))
}

pub fn chats_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_chats))
        .route("/", post(create_chat))
        .route("/{id}", get(get_chat).delete(delete_chat))
        .route("/{id}/title", axum::routing::patch(update_chat_title))
}