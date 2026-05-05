use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
};
use serde_json::json;

use crate::{
    dtos::{
        ui_state::{ListScopeQuery, UpsertUiStateDto},
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::ui_state::UiStateService,
    state::AppState,
};

const SCOPE_MAX_LEN: usize = 40;
const KEY_MAX_LEN: usize = 120;

fn ensure_valid_scope(scope: &str) -> Result<(), AppError> {
    if scope.is_empty() || scope.len() > SCOPE_MAX_LEN {
        return Err(AppError::BadRequest("invalid ui state scope".to_string()));
    }

    let is_valid = scope
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !is_valid {
        return Err(AppError::BadRequest("invalid ui state scope".to_string()));
    }

    Ok(())
}

fn ensure_valid_key(key: &str) -> Result<(), AppError> {
    if key.is_empty() || key.len() > KEY_MAX_LEN {
        return Err(AppError::BadRequest("invalid ui state key".to_string()));
    }

    let is_valid = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':');

    if !is_valid {
        return Err(AppError::BadRequest("invalid ui state key".to_string()));
    }

    Ok(())
}

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
    let entries = UiStateService::list_scope(&state, user.id, &query.scope).await?;
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

    match UiStateService::get_one(&state, user.id, &scope, &key).await? {
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

    let entry = UiStateService::upsert(&state, user.id, &scope, &key, payload.value).await?;
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

    UiStateService::delete_one(&state, user.id, &scope, &key).await?;
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

    UiStateService::delete_scope(&state, user.id, &scope).await?;
    Ok(Json(json!({ "message": "Scope cleared" })))
}

pub fn ui_state_controller() -> Router<AppState> {
    Router::new()
        .route("/", get(list_scope))
        .route("/{scope}", delete(delete_scope))
        .route("/{scope}/{key}", get(get_one).put(upsert).delete(delete_one))
}
