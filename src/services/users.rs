use axum::{extract::FromRequestParts, http::request::Parts};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use uuid::Uuid;

use crate::utils::crypto::{compare_password, hash_password};
use crate::{
    dtos::user::{
        UpdateNotificationPreferencesDto, UpdateUserPasswordDto, UpdateUserPreferencesDto, UpdateUserProfileDto,
        UserPreferences, UserSettingsDto,
    },
    entities::{
        prelude::Users,
        users::{self, Model as UserModel},
    },
    env::Config,
    error::AppError,
    state::AppState,
    utils::crypto::encrypt_secret,
};

#[derive(Clone)]
pub struct UserService {
    pub db: Arc<DatabaseConnection>,
    pub config: Config,
}

impl UserService {
    pub fn new(db: Arc<DatabaseConnection>, config: Config) -> Self {
        Self { db, config }
    }

    async fn get_user_model(&self, user_id: Uuid) -> Result<UserModel, AppError> {
        Users::find_by_id(user_id)
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::Unauthorized("Usuario no autenticado".to_string()))
    }

    fn serialize_preferences(preferences: &UserPreferences) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(preferences)
            .map_err(|_| AppError::Internal("No se pudieron serializar las preferencias".to_string()))
    }

    pub async fn get_me(&self, user_id: Uuid) -> Result<UserSettingsDto, AppError> {
        let user = self.get_user_model(user_id).await?;
        Ok(user.into())
    }

    pub async fn update_profile(&self, user_id: Uuid, dto: UpdateUserProfileDto) -> Result<UserSettingsDto, AppError> {
        let normalized_name = dto.name.trim().to_string();
        let normalized_email = dto.email.trim().to_lowercase();

        if normalized_name.is_empty() {
            return Err(AppError::BadRequest("El nombre no puede estar vacío".to_string()));
        }

        let email_exists = Users::find()
            .filter(users::Column::Email.eq(normalized_email.clone()))
            .filter(users::Column::Id.ne(user_id))
            .one(self.db.as_ref())
            .await?;

        if email_exists.is_some() {
            return Err(AppError::Conflict("Ese correo ya está en uso".to_string()));
        }

        let name_exists = Users::find()
            .filter(users::Column::Name.eq(normalized_name.clone()))
            .filter(users::Column::Id.ne(user_id))
            .one(self.db.as_ref())
            .await?;

        if name_exists.is_some() {
            return Err(AppError::Conflict("Ese nombre ya está en uso".to_string()));
        }

        let user = self.get_user_model(user_id).await?;
        let mut active_user: users::ActiveModel = user.into();
        active_user.name = Set(normalized_name);
        active_user.email = Set(normalized_email);

        let updated = active_user.update(self.db.as_ref()).await?;
        Ok(updated.into())
    }

    pub async fn update_password(&self, user_id: Uuid, dto: UpdateUserPasswordDto) -> Result<(), AppError> {
        let user = self.get_user_model(user_id).await?;

        let _ = compare_password(dto.current_password, user.password_hash.clone())?;
        let new_password_hash = hash_password(dto.new_password)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.password_hash = Set(new_password_hash);
        active_user.update(self.db.as_ref()).await?;

        Ok(())
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        dto: UpdateUserPreferencesDto,
    ) -> Result<UserSettingsDto, AppError> {
        let user = self.get_user_model(user_id).await?;
        let mut preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        preferences.apply_general_patch(&dto);

        if let Some(api_key) = dto.ai_api_key.as_ref() {
            let cleaned_key = api_key.trim();
            if cleaned_key.is_empty() {
                return Err(AppError::BadRequest("La API key AI no puede estar vacia".to_string()));
            }

            preferences.ai.api_key_encrypted = Some(encrypt_secret(&self.config.ai_prefs_encryption_key, cleaned_key)?);
            preferences.ai.api_key_configured = true;
        }

        let preferences_json = Self::serialize_preferences(&preferences)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.preferences = Set(Some(preferences_json));

        let updated = active_user.update(self.db.as_ref()).await?;
        Ok(updated.into())
    }

    pub async fn update_notifications(
        &self,
        user_id: Uuid,
        dto: UpdateNotificationPreferencesDto,
    ) -> Result<UserSettingsDto, AppError> {
        let user = self.get_user_model(user_id).await?;
        let mut preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        preferences.apply_notifications_patch(dto);

        let preferences_json = Self::serialize_preferences(&preferences)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.preferences = Set(Some(preferences_json));

        let updated = active_user.update(self.db.as_ref()).await?;
        Ok(updated.into())
    }
}

impl FromRequestParts<AppState> for UserService {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(UserService::new(state.db.clone(), state.config.clone()))
    }
}
