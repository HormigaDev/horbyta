use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, sea_query::OnConflict};
use uuid::Uuid;

use crate::{
    dtos::ui_state::{UiStateEntryDto, assert_value_within_limit},
    entities::{prelude::UserUiState, user_ui_state},
    error::AppError,
    state::AppState,
};

pub struct UiStateService;

impl UiStateService {
    fn cache_key(user_id: Uuid, scope: &str) -> String {
        format!("{}:{}", user_id, scope)
    }

    pub async fn list_scope(state: &AppState, user_id: Uuid, scope: &str) -> Result<Vec<UiStateEntryDto>, AppError> {
        let key = Self::cache_key(user_id, scope);

        if let Some(cached) = state.ui_state_cache.get(&key).await {
            return Ok(cached);
        }

        let rows = UserUiState::find()
            .filter(user_ui_state::Column::UserId.eq(user_id))
            .filter(user_ui_state::Column::Scope.eq(scope.to_string()))
            .order_by_asc(user_ui_state::Column::Key)
            .all(state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        let entries: Vec<UiStateEntryDto> = rows.into_iter().map(UiStateEntryDto::from).collect();

        state.ui_state_cache.insert(key, entries.clone()).await;
        Ok(entries)
    }

    pub async fn get_one(
        state: &AppState,
        user_id: Uuid,
        scope: &str,
        key: &str,
    ) -> Result<Option<UiStateEntryDto>, AppError> {
        let entries = Self::list_scope(state, user_id, scope).await?;
        Ok(entries.into_iter().find(|entry| entry.key == key))
    }

    pub async fn upsert(
        state: &AppState,
        user_id: Uuid,
        scope: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<UiStateEntryDto, AppError> {
        assert_value_within_limit(&value)?;

        let active = user_ui_state::ActiveModel {
            user_id: Set(user_id),
            scope: Set(scope.to_string()),
            key: Set(key.to_string()),
            value: Set(value),
            updated_at: Set(Utc::now().fixed_offset()),
        };

        UserUiState::insert(active)
            .on_conflict(
                OnConflict::columns([
                    user_ui_state::Column::UserId,
                    user_ui_state::Column::Scope,
                    user_ui_state::Column::Key,
                ])
                .update_columns([user_ui_state::Column::Value, user_ui_state::Column::UpdatedAt])
                .to_owned(),
            )
            .exec(state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        let row = UserUiState::find_by_id((user_id, scope.to_string(), key.to_string()))
            .one(state.db.as_ref())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Internal("ui state row missing after upsert".to_string()))?;

        let entry: UiStateEntryDto = row.into();
        Self::refresh_scope_cache(state, user_id, scope, &entry).await;
        Ok(entry)
    }

    pub async fn delete_one(state: &AppState, user_id: Uuid, scope: &str, key: &str) -> Result<(), AppError> {
        UserUiState::delete_by_id((user_id, scope.to_string(), key.to_string()))
            .exec(state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        Self::evict_scope(state, user_id, scope).await;
        Ok(())
    }

    pub async fn delete_scope(state: &AppState, user_id: Uuid, scope: &str) -> Result<(), AppError> {
        UserUiState::delete_many()
            .filter(user_ui_state::Column::UserId.eq(user_id))
            .filter(user_ui_state::Column::Scope.eq(scope.to_string()))
            .exec(state.db.as_ref())
            .await
            .map_err(AppError::Database)?;

        Self::evict_scope(state, user_id, scope).await;
        Ok(())
    }

    async fn refresh_scope_cache(state: &AppState, user_id: Uuid, scope: &str, updated: &UiStateEntryDto) {
        let cache_key = Self::cache_key(user_id, scope);
        let Some(mut entries) = state.ui_state_cache.get(&cache_key).await else {
            return;
        };

        if let Some(existing) = entries.iter_mut().find(|item| item.key == updated.key) {
            *existing = updated.clone();
        } else {
            entries.push(updated.clone());
            entries.sort_by(|a, b| a.key.cmp(&b.key));
        }

        state.ui_state_cache.insert(cache_key, entries).await;
    }

    async fn evict_scope(state: &AppState, user_id: Uuid, scope: &str) {
        state.ui_state_cache.invalidate(&Self::cache_key(user_id, scope)).await;
    }

    pub async fn invalidate_user(state: &AppState, user_id: Uuid) {
        let prefix = format!("{}:", user_id);
        state
            .ui_state_cache
            .invalidate_entries_if(move |key, _| key.starts_with(&prefix))
            .ok();
    }
}
