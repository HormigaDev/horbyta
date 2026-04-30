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
        category::{CreateCategoryDto, UpdateCategoryDto, category_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::categories::CategoryService,
    state::AppState,
};

async fn search_categories(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    category_filter_schema().validate_payload(&payload)?;

    let (categories, total_items) = CategoryService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": categories,
        "total": total_items
    })))
}

async fn get_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(category))
}

async fn create_category(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(category))
}

async fn update_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(category))
}

async fn delete_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    CategoryService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Category deleted" })))
}

pub fn categories_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_categories))
        .route("/", post(create_category))
        .route("/{id}", get(get_category).put(update_category).delete(delete_category))
}
