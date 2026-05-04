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

#[utoipa::path(
    get,
    path = "/api/v1/categories/search",
    tag = "Categories",
    params(QueryPayload),
    responses(
        (status = 200, description = "Paginated list of categories"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_categories(
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

#[utoipa::path(
    get,
    path = "/api/v1/categories/{id}",
    tag = "Categories",
    params(("id" = Uuid, Path, description = "Category id")),
    responses(
        (status = 200, description = "Category detail", body = crate::dtos::category::CategoryDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn get_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(category))
}

#[utoipa::path(
    post,
    path = "/api/v1/categories/",
    tag = "Categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 200, description = "Category created", body = crate::dtos::category::CategoryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_category(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(category))
}

#[utoipa::path(
    put,
    path = "/api/v1/categories/{id}",
    tag = "Categories",
    params(("id" = Uuid, Path, description = "Category id")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Category updated", body = crate::dtos::category::CategoryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn update_category(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCategoryDto>,
) -> Result<impl IntoResponse, AppError> {
    let category = CategoryService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(category))
}

#[utoipa::path(
    delete,
    path = "/api/v1/categories/{id}",
    tag = "Categories",
    params(("id" = Uuid, Path, description = "Category id")),
    responses(
        (status = 200, description = "Category deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Not found")
    )
)]
pub(crate) async fn delete_category(
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
