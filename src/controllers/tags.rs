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
        query::QueryPayload,
        tag::{CreateTagDto, UpdateTagDto, tag_filter_schema},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::tags::TagService,
    state::AppState,
};

async fn search_tags(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    tag_filter_schema().validate_payload(&payload)?;

    let (tags, total_items) = TagService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": tags,
        "total": total_items
    })))
}

async fn get_tag(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(tag))
}

async fn create_tag(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateTagDto>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(tag))
}

async fn update_tag(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTagDto>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(tag))
}

async fn delete_tag(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    TagService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Etiqueta eliminada" })))
}

pub fn tags_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_tags))
        .route("/", post(create_tag))
        .route("/{id}", get(get_tag).put(update_tag).delete(delete_tag))
}
