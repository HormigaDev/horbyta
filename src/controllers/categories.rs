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
        category::{CategoryDto, category_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::categories::CategoryService,
    state::AppState,
};

async fn search_categories(
    user: UserDto,
    service: CategoryService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    category_filter_schema().validate_payload(&payload)?;

    let (categories, total_items) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": categories,
        "total": total_items
    })))
}

async fn get_category(
    user: UserDto,
    service: CategoryService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let category = service.get_by_id(user.id, id).await?;
    Ok(Json(category))
}

async fn create_category(
    user: UserDto,
    service: CategoryService,
    ValidatedJson(payload): ValidatedJson<CategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = service.create(user.id, payload).await?;
    Ok(Json(category))
}

async fn update_category(
    user: UserDto,
    service: CategoryService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<CategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = service.update(user.id, id, payload).await?;
    Ok(Json(category))
}

async fn delete_category(
    user: UserDto,
    service: CategoryService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({ "message": "Categoría eliminada" })))
}

pub fn categories_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_categories))
        .route("/", post(create_category))
        .route("/{id}", get(get_category).put(update_category).delete(delete_category))
}
