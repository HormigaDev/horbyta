use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    dtos::{ui_state::UpsertUiStateDto, user::UserDto},
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

    let is_valid = key.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'
    });

    if !is_valid {
        return Err(AppError::BadRequest("invalid ui state key".to_string()));
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ListScopeQuery {
    scope: String,
}

async fn list_scope(
    State(state): State<AppState>,
    user: UserDto,
    axum::extract::Query(query): axum::extract::Query<ListScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&query.scope)?;
    let entries = UiStateService::list_scope(&state, user.id, &query.scope).await?;
    Ok(Json(json!({ "items": entries })))
}

async fn get_one(
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

async fn upsert(
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

async fn delete_one(
    State(state): State<AppState>,
    user: UserDto,
    Path((scope, key)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_valid_scope(&scope)?;
    ensure_valid_key(&key)?;

    UiStateService::delete_one(&state, user.id, &scope, &key).await?;
    Ok(Json(json!({ "message": "Entry deleted" })))
}

async fn delete_scope(
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
