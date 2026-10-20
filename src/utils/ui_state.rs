use uuid::Uuid;

use crate::error_codes as codes;
use crate::{dtos::ui_state::UiStateEntryDto, error::AppError, services::ui_state::UiStateService, state::AppState};

const SCOPE_MAX_LEN: usize = 40;
const KEY_MAX_LEN: usize = 120;

pub fn ensure_valid_scope(scope: &str) -> Result<(), AppError> {
    if scope.is_empty() || scope.len() > SCOPE_MAX_LEN {
        return Err(AppError::BadRequest(codes::UI_STATE_SCOPE_INVALID.into()));
    }

    let is_valid = scope
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    if !is_valid {
        return Err(AppError::BadRequest(codes::UI_STATE_SCOPE_INVALID.into()));
    }

    Ok(())
}

pub fn ensure_valid_key(key: &str) -> Result<(), AppError> {
    if key.is_empty() || key.len() > KEY_MAX_LEN {
        return Err(AppError::BadRequest(codes::UI_STATE_KEY_INVALID.into()));
    }

    let is_valid = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':');

    if !is_valid {
        return Err(AppError::BadRequest(codes::UI_STATE_KEY_INVALID.into()));
    }

    Ok(())
}

fn cache_key(user_id: Uuid, scope: &str) -> String {
    format!("{}:{}", user_id, scope)
}

pub async fn list_scope_cached(state: &AppState, user_id: Uuid, scope: &str) -> Result<Vec<UiStateEntryDto>, AppError> {
    let key = cache_key(user_id, scope);

    if let Some(cached) = state.ui_state_cache.get(&key).await {
        return Ok(cached);
    }

    let entries = UiStateService::list_scope(state.db.as_ref(), user_id, scope).await?;
    state.ui_state_cache.insert(key, entries.clone()).await;
    Ok(entries)
}

pub async fn refresh_scope_cache(state: &AppState, user_id: Uuid, scope: &str, updated: &UiStateEntryDto) {
    let key = cache_key(user_id, scope);
    let Some(mut entries) = state.ui_state_cache.get(&key).await else {
        return;
    };

    if let Some(existing) = entries.iter_mut().find(|item| item.key == updated.key) {
        *existing = updated.clone();
    } else {
        entries.push(updated.clone());
        entries.sort_by(|a, b| a.key.cmp(&b.key));
    }

    state.ui_state_cache.insert(key, entries).await;
}

pub async fn evict_scope(state: &AppState, user_id: Uuid, scope: &str) {
    state.ui_state_cache.invalidate(&cache_key(user_id, scope)).await;
}
