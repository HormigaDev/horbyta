use chrono::Utc;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set, sea_query::OnConflict};
use uuid::Uuid;

use crate::{
    dtos::ui_state::{UiStateEntryDto, assert_value_within_limit},
    entities::{prelude::UserUiState, user_ui_state},
    error::AppError,
};

pub struct UiStateService;

impl UiStateService {
    pub async fn list_scope(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        scope: &str,
    ) -> Result<Vec<UiStateEntryDto>, AppError> {
        let rows = UserUiState::find()
            .filter(user_ui_state::Column::UserId.eq(user_id))
            .filter(user_ui_state::Column::Scope.eq(scope.to_string()))
            .order_by_asc(user_ui_state::Column::Key)
            .all(db)
            .await
            .map_err(AppError::Database)?;

        Ok(rows.into_iter().map(UiStateEntryDto::from).collect())
    }

    pub async fn get_one(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        scope: &str,
        key: &str,
    ) -> Result<Option<UiStateEntryDto>, AppError> {
        let entries = Self::list_scope(db, user_id, scope).await?;
        Ok(entries.into_iter().find(|entry| entry.key == key))
    }

    pub async fn upsert(
        db: &impl ConnectionTrait,
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
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        let row = UserUiState::find_by_id((user_id, scope.to_string(), key.to_string()))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Internal("ui state row missing after upsert".to_string()))?;

        Ok(row.into())
    }

    pub async fn delete_one(db: &impl ConnectionTrait, user_id: Uuid, scope: &str, key: &str) -> Result<(), AppError> {
        UserUiState::delete_by_id((user_id, scope.to_string(), key.to_string()))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn delete_scope(db: &impl ConnectionTrait, user_id: Uuid, scope: &str) -> Result<(), AppError> {
        UserUiState::delete_many()
            .filter(user_ui_state::Column::UserId.eq(user_id))
            .filter(user_ui_state::Column::Scope.eq(scope.to_string()))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn batch_upsert(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        scope: &str,
        entries: Vec<(String, serde_json::Value)>,
    ) -> Result<Vec<UiStateEntryDto>, AppError> {
        let mut results = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let entry = Self::upsert(db, user_id, scope, &key, value).await?;
            results.push(entry);
        }
        Ok(results)
    }
}
