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

#[utoipa::path(
    get,
    path = "/api/v1/tags/search",
    tag = "Tags",
    params(crate::dtos::query::SearchQuery),
    responses(
        (status = 200, description = "Paginated list of tags"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_tags(
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

#[utoipa::path(
    get,
    path = "/api/v1/tags/{id}",
    tag = "Tags",
    params(("id" = Uuid, Path, description = "Tag id")),
    responses(
        (status = 200, description = "Tag detail", body = crate::dtos::tag::TagDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Tag not found")
    )
)]
pub(crate) async fn get_tag(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(tag))
}

#[utoipa::path(
    post,
    path = "/api/v1/tags/",
    tag = "Tags",
    request_body = CreateTagDto,
    responses(
        (status = 200, description = "Tag created", body = crate::dtos::tag::TagDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_tag(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateTagDto>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(tag))
}

#[utoipa::path(
    put,
    path = "/api/v1/tags/{id}",
    tag = "Tags",
    params(("id" = Uuid, Path, description = "Tag id")),
    request_body = UpdateTagDto,
    responses(
        (status = 200, description = "Tag updated", body = crate::dtos::tag::TagDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Tag not found")
    )
)]
pub(crate) async fn update_tag(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTagDto>,
) -> Result<impl IntoResponse, AppError> {
    let tag = TagService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(tag))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{id}",
    tag = "Tags",
    params(("id" = Uuid, Path, description = "Tag id")),
    responses(
        (status = 200, description = "Tag deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Tag not found")
    )
)]
pub(crate) async fn delete_tag(
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
