use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
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
    error::AppError,
};

pub struct UserService;

impl UserService {
    async fn get_user_model(db: &impl ConnectionTrait, user_id: Uuid) -> Result<UserModel, AppError> {
        Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not authenticated".to_string()))
    }

    fn serialize_preferences(preferences: &UserPreferences) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(preferences).map_err(|_| AppError::Internal("could not serialize preferences".to_string()))
    }

    pub async fn get_me(db: &impl ConnectionTrait, user_id: Uuid) -> Result<UserSettingsDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        Ok(user.into())
    }

    pub async fn update_profile(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: UpdateUserProfileDto,
    ) -> Result<UserSettingsDto, AppError> {
        let normalized_name = dto.name.trim().to_string();
        let normalized_email = dto.email.trim().to_lowercase();

        if normalized_name.is_empty() {
            return Err(AppError::BadRequest("name cannot be empty".to_string()));
        }

        let email_exists = Users::find()
            .filter(users::Column::Email.eq(normalized_email.clone()))
            .filter(users::Column::Id.ne(user_id))
            .one(db)
            .await?;

        if email_exists.is_some() {
            return Err(AppError::Conflict("that email is already in use".to_string()));
        }

        let name_exists = Users::find()
            .filter(users::Column::Name.eq(normalized_name.clone()))
            .filter(users::Column::Id.ne(user_id))
            .one(db)
            .await?;

        if name_exists.is_some() {
            return Err(AppError::Conflict("that name is already in use".to_string()));
        }

        let user = Self::get_user_model(db, user_id).await?;
        let mut active_user: users::ActiveModel = user.into();
        active_user.name = Set(normalized_name);
        active_user.email = Set(normalized_email);

        let updated = active_user.update(db).await?;
        Ok(updated.into())
    }

    pub async fn update_password(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: UpdateUserPasswordDto,
    ) -> Result<(), AppError> {
        let user = Self::get_user_model(db, user_id).await?;

        let _ = compare_password(dto.current_password, user.password_hash.clone())?;
        let new_password_hash = hash_password(dto.new_password)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.password_hash = Set(new_password_hash);
        active_user.update(db).await?;

        Ok(())
    }

    pub async fn update_preferences(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: UpdateUserPreferencesDto,
    ) -> Result<UserSettingsDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        let mut preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        preferences.apply_general_patch(&dto);

        let preferences_json = Self::serialize_preferences(&preferences)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.preferences = Set(Some(preferences_json));

        let updated = active_user.update(db).await?;
        Ok(updated.into())
    }

    pub async fn update_notifications(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: UpdateNotificationPreferencesDto,
    ) -> Result<UserSettingsDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        let mut preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        preferences.apply_notifications_patch(dto);

        let preferences_json = Self::serialize_preferences(&preferences)?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.preferences = Set(Some(preferences_json));

        let updated = active_user.update(db).await?;
        Ok(updated.into())
    }

}
