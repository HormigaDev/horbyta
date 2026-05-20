use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
};
use sea_orm::TransactionTrait;
use serde_json::json;

use crate::utils::ui_state::{
    ensure_valid_key, ensure_valid_scope, evict_scope, list_scope_cached, refresh_scope_cache,
};
use crate::{
    dtos::{
        ui_state::{BatchUpsertUiStateDto, ListScopeQuery, UpsertUiStateDto},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::ui_state::UiStateService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/users/me/ui-state/",
    tag = "UI State",
    params(ListScopeQuery),
    responses(
        (status = 200, description = "UI state entries for scope"),
        (status = 400, description = "Invalid scope"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn list_scope(
    State(state): State<AppState>,
    user: UserDto,
    axum::extract::Query(query): axum::extract::Query<ListScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&query.scope)?;
    let entries = list_scope_cached(&state, user.id, &query.scope).await?;
    Ok(Json(json!({ "items": entries })))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/ui-state/{scope}/{key}",
    tag = "UI State",
    params(
        ("scope" = String, Path, description = "UI state scope"),
        ("key" = String, Path, description = "UI state key")
    ),
    responses(
        (status = 200, description = "UI state entry", body = crate::dtos::ui_state::UiStateEntryDto),
        (status = 400, description = "Invalid scope or key"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "UI state entry not found")
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    user: UserDto,
    Path((scope, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;
    ensure_valid_key(&key)?;

    let entries = list_scope_cached(&state, user.id, &scope).await?;
    match entries.into_iter().find(|entry| entry.key == key) {
        Some(entry) => Ok(Json(entry).into_response()),
        None => Err(AppError::NotFound("ui state entry not found".to_string())),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/users/me/ui-state/{scope}/{key}",
    tag = "UI State",
    params(
        ("scope" = String, Path, description = "UI state scope"),
        ("key" = String, Path, description = "UI state key")
    ),
    request_body = UpsertUiStateDto,
    responses(
        (status = 200, description = "UI state entry upserted", body = crate::dtos::ui_state::UiStateEntryDto),
        (status = 400, description = "Invalid scope, key, or payload"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn upsert(
    State(state): State<AppState>,
    user: UserDto,
    Path((scope, key)): Path<(String, String)>,
    ValidatedJson(payload): ValidatedJson<UpsertUiStateDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;
    ensure_valid_key(&key)?;

    let entry = UiStateService::upsert(state.db.as_ref(), user.id, &scope, &key, payload.value).await?;
    refresh_scope_cache(&state, user.id, &scope, &entry).await;
    Ok(Json(entry))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/me/ui-state/{scope}/{key}",
    tag = "UI State",
    params(
        ("scope" = String, Path, description = "UI state scope"),
        ("key" = String, Path, description = "UI state key")
    ),
    responses(
        (status = 200, description = "UI state entry deleted"),
        (status = 400, description = "Invalid scope or key"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    user: UserDto,
    Path((scope, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;
    ensure_valid_key(&key)?;

    UiStateService::delete_one(state.db.as_ref(), user.id, &scope, &key).await?;
    evict_scope(&state, user.id, &scope).await;
    Ok(Json(json!({ "message": "Entry deleted" })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/me/ui-state/{scope}",
    tag = "UI State",
    params(("scope" = String, Path, description = "UI state scope")),
    responses(
        (status = 200, description = "UI state scope cleared"),
        (status = 400, description = "Invalid scope"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn delete_scope(
    State(state): State<AppState>,
    user: UserDto,
    Path(scope): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;

    UiStateService::delete_scope(state.db.as_ref(), user.id, &scope).await?;
    evict_scope(&state, user.id, &scope).await;
    Ok(Json(json!({ "message": "Scope cleared" })))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/me/ui-state/{scope}/batch",
    tag = "UI State",
    params(("scope" = String, Path, description = "UI state scope")),
    request_body = BatchUpsertUiStateDto,
    responses(
        (status = 200, description = "Batch upsert result"),
        (status = 400, description = "Invalid scope, key, or payload"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn batch_upsert(
    State(state): State<AppState>,
    user: UserDto,
    Path(scope): Path<String>,
    ValidatedJson(payload): ValidatedJson<BatchUpsertUiStateDto>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;

    let entries: Result<Vec<(String, serde_json::Value)>, AppError> = payload
        .entries
        .into_iter()
        .map(|e| {
            ensure_valid_key(&e.key)?;
            Ok((e.key, e.value))
        })
        .collect();

    let tx = state.db.begin().await.map_err(AppError::Database)?;
    let updated = UiStateService::batch_upsert(&tx, user.id, &scope, entries?).await?;
    tx.commit().await.map_err(AppError::Database)?;
    for entry in &updated {
        refresh_scope_cache(&state, user.id, &scope, entry).await;
    }
    Ok(Json(json!({ "items": updated })))
}

pub fn ui_state_controller() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scope))
        .route("/{scope}", delete(delete_scope))
        .route("/{scope}/batch", axum::routing::put(batch_upsert))
        .route("/{scope}/{key}", get(get_one).put(upsert).delete(delete_one))
}
