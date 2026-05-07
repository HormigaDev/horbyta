use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    auth::totp_2fa::{
        TotpPolicy, backup_codes_file_content, generate_backup_codes, generate_setup, hash_backup_codes,
        verify_backup_code, verify_code,
    },
    dtos::auth::RegisterPayload,
    dtos::query::QueryPayload,
    dtos::totp_2fa::{DisableTotpDto, TotpEnableDto, TotpSetupDto, TotpStatusDto, VerifyTotpCodeDto},
    dtos::user::{
        AdminCreateUserDto, AdminUpdateUserDto, AdminUserSummaryDto, UpdateNotificationPreferencesDto,
        UpdateUserPasswordDto, UpdateUserPreferencesDto, UpdateUserProfileDto, UserPreferences, UserRole,
        UserSearchField, UserSettingsDto, UserStatus,
    },
    entities::{
        prelude::Users,
        users::{self, Model as UserModel},
    },
    error::{AppError, Totp2FaError},
    utils::{
        crypto::{compare_password, hash_password},
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub struct UserService;

struct NewUserInput {
    name: String,
    email: String,
    password: String,
    role: UserRole,
    status: UserStatus,
}

impl UserService {
    async fn get_user_model(db: &impl ConnectionTrait, user_id: Uuid) -> Result<UserModel, AppError> {
        Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not authenticated".to_string()))
    }

    async fn find_user_by_id(db: &impl ConnectionTrait, user_id: Uuid) -> Result<UserModel, AppError> {
        Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))
    }

    async fn ensure_admin(db: &impl ConnectionTrait, user_id: Uuid) -> Result<(), AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        let role = UserRole::from_str(&user.role)
            .map_err(|_| AppError::Forbidden("admin privileges are required".to_string()))?;

        if role != UserRole::Admin {
            return Err(AppError::Forbidden("admin privileges are required".to_string()));
        }

        Ok(())
    }

    pub fn ensure_user_can_authenticate(user: &UserModel) -> Result<(), AppError> {
        match UserStatus::from_str(&user.status).unwrap_or(UserStatus::Inactive) {
            UserStatus::Active => Ok(()),
            UserStatus::Inactive => Err(AppError::Unauthorized("invalid credentials".to_string())),
            UserStatus::Blocked => Err(AppError::Forbidden("account blocked".to_string())),
        }
    }

    fn normalize_name(name: &str) -> Result<String, AppError> {
        let normalized_name = name.trim().to_string();
        if normalized_name.is_empty() {
            return Err(AppError::BadRequest("name cannot be empty".to_string()));
        }

        Ok(normalized_name)
    }

    fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
    }

    async fn ensure_email_available(
        db: &impl ConnectionTrait,
        email: &str,
        excluded_user_id: Option<Uuid>,
        message: &str,
    ) -> Result<(), AppError> {
        let mut query = Users::find().filter(users::Column::Email.eq(email.to_string()));
        if let Some(user_id) = excluded_user_id {
            query = query.filter(users::Column::Id.ne(user_id));
        }

        if query.one(db).await?.is_some() {
            return Err(AppError::Conflict(message.to_string()));
        }

        Ok(())
    }

    async fn ensure_name_available(
        db: &impl ConnectionTrait,
        name: &str,
        excluded_user_id: Option<Uuid>,
        message: &str,
    ) -> Result<(), AppError> {
        let mut query = Users::find().filter(users::Column::Name.eq(name.to_string()));
        if let Some(user_id) = excluded_user_id {
            query = query.filter(users::Column::Id.ne(user_id));
        }

        if query.one(db).await?.is_some() {
            return Err(AppError::Conflict(message.to_string()));
        }

        Ok(())
    }

    async fn create_user(db: &impl ConnectionTrait, input: NewUserInput) -> Result<UserModel, AppError> {
        let normalized_name = Self::normalize_name(&input.name)?;
        let normalized_email = Self::normalize_email(&input.email);

        Self::ensure_email_available(db, &normalized_email, None, "email is already registered").await?;
        Self::ensure_name_available(db, &normalized_name, None, "username is already taken").await?;

        let password_hash = hash_password(input.password)?;
        let now = Utc::now().fixed_offset();

        let new_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(normalized_name),
            email: Set(normalized_email),
            password_hash: Set(password_hash),
            role: Set(input.role.to_string()),
            status: Set(input.status.to_string()),
            preferences: Set(None),
            two_factor_auth_enabled: Set(false),
            two_factor_auth_secret_base32: Set(None),
            two_factor_auth_pending_secret_base32: Set(None),
            two_factor_auth_last_used_step: Set(None),
            two_factor_auth_backup_code_hashes: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        Ok(new_user.insert(db).await?)
    }

    fn serialize_preferences(preferences: &UserPreferences) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(preferences).map_err(|_| AppError::Internal("could not serialize preferences".to_string()))
    }

    fn serialize_backup_code_hashes(hashes: &[String]) -> Result<serde_json::Value, AppError> {
        serde_json::to_value(hashes)
            .map_err(|_| AppError::Internal("could not serialize backup code hashes".to_string()))
    }

    fn backup_code_hashes_from_json(raw: Option<&serde_json::Value>) -> Vec<String> {
        raw.and_then(|value| serde_json::from_value::<Vec<String>>(value.clone()).ok())
            .unwrap_or_default()
    }

    pub async fn get_me(db: &impl ConnectionTrait, user_id: Uuid) -> Result<UserSettingsDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        Ok(user.into())
    }

    pub async fn register(db: &impl ConnectionTrait, dto: RegisterPayload) -> Result<UserModel, AppError> {
        Self::create_user(
            db,
            NewUserInput {
                name: dto.name,
                email: dto.email,
                password: dto.password,
                role: UserRole::User,
                status: UserStatus::Active,
            },
        )
        .await
    }

    pub async fn admin_search_users(
        db: &impl ConnectionTrait,
        admin_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<AdminUserSummaryDto>, u64), AppError> {
        Self::ensure_admin(db, admin_id).await?;

        let condition = apply_filters(Condition::all(), &payload.filters, |column| {
            column
                .parse::<UserSearchField>()
                .ok()
                .map(UserSearchField::filter_field)
        })?;

        let query = Users::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            UserSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<UserSearchField>()
                    .ok()
                    .map(UserSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let models = apply_pagination(
            query
                .select_only()
                .column(users::Column::Id)
                .column(users::Column::Name)
                .column(users::Column::Email)
                .column(users::Column::Role)
                .column(users::Column::Status)
                .column(users::Column::CreatedAt)
                .column(users::Column::UpdatedAt),
            &payload,
        )
        .into_model::<AdminUserSummaryDto>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    pub async fn admin_create_user(
        db: &impl ConnectionTrait,
        admin_id: Uuid,
        dto: AdminCreateUserDto,
    ) -> Result<UserSettingsDto, AppError> {
        Self::ensure_admin(db, admin_id).await?;

        let user = Self::create_user(
            db,
            NewUserInput {
                name: dto.name,
                email: dto.email,
                password: dto.password,
                role: dto.role,
                status: dto.status,
            },
        )
        .await?;

        Ok(user.into())
    }

    pub async fn admin_update_user(
        db: &impl ConnectionTrait,
        admin_id: Uuid,
        target_user_id: Uuid,
        dto: AdminUpdateUserDto,
    ) -> Result<UserSettingsDto, AppError> {
        Self::ensure_admin(db, admin_id).await?;

        let user = Self::find_user_by_id(db, target_user_id).await?;
        let mut active_user: users::ActiveModel = user.into();

        if let Some(name) = dto.name {
            let normalized_name = Self::normalize_name(&name)?;
            Self::ensure_name_available(
                db,
                &normalized_name,
                Some(target_user_id),
                "that name is already in use",
            )
            .await?;
            active_user.name = Set(normalized_name);
        }

        if let Some(email) = dto.email {
            let normalized_email = Self::normalize_email(&email);
            Self::ensure_email_available(
                db,
                &normalized_email,
                Some(target_user_id),
                "that email is already in use",
            )
            .await?;
            active_user.email = Set(normalized_email);
        }

        if let Some(password) = dto.password {
            active_user.password_hash = Set(hash_password(password)?);
        }

        if let Some(role) = dto.role {
            if admin_id == target_user_id && role != UserRole::Admin {
                return Err(AppError::BadRequest(
                    "admin users cannot remove their own admin role".to_string(),
                ));
            }
            active_user.role = Set(role.to_string());
        }

        if let Some(status) = dto.status {
            if admin_id == target_user_id && status != UserStatus::Active {
                return Err(AppError::BadRequest(
                    "admin users cannot deactivate or block themselves".to_string(),
                ));
            }
            active_user.status = Set(status.to_string());
        }

        active_user.updated_at = Set(Utc::now().fixed_offset());
        let updated = active_user.update(db).await?;

        Ok(updated.into())
    }

    pub async fn admin_delete_user(
        db: &impl ConnectionTrait,
        admin_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), AppError> {
        Self::ensure_admin(db, admin_id).await?;
        if admin_id == target_user_id {
            return Err(AppError::BadRequest("admin users cannot delete themselves".to_string()));
        }

        let user = Self::find_user_by_id(db, target_user_id).await?;
        users::ActiveModel {
            id: Set(user.id),
            ..Default::default()
        }
        .delete(db)
        .await?;

        Ok(())
    }

    pub async fn update_profile(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: UpdateUserProfileDto,
    ) -> Result<UserSettingsDto, AppError> {
        let normalized_name = Self::normalize_name(&dto.name)?;
        let normalized_email = Self::normalize_email(&dto.email);

        Self::ensure_email_available(db, &normalized_email, Some(user_id), "that email is already in use").await?;
        Self::ensure_name_available(db, &normalized_name, Some(user_id), "that name is already in use").await?;

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
        policy: &TotpPolicy,
        dto: UpdateUserPasswordDto,
    ) -> Result<(), AppError> {
        let user = Self::get_user_model(db, user_id).await?;

        compare_password(dto.current_password, user.password_hash.clone())?;
        let new_password_hash = hash_password(dto.new_password)?;

        let mut active_user: users::ActiveModel = user.clone().into();
        if user.two_factor_auth_enabled {
            Self::apply_second_factor_verification(
                &user,
                &mut active_user,
                policy,
                dto.two_factor_auth_code.as_deref(),
                dto.two_factor_auth_backup_code.as_deref(),
            )?;
        }

        active_user.password_hash = Set(new_password_hash);
        active_user.updated_at = Set(Utc::now().fixed_offset());
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

    pub async fn get_totp_status(db: &impl ConnectionTrait, user_id: Uuid) -> Result<TotpStatusDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        Ok(Self::to_totp_status(&user))
    }

    pub async fn start_totp_setup(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        policy: &TotpPolicy,
    ) -> Result<TotpSetupDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        if user.two_factor_auth_enabled {
            return Err(Totp2FaError::AlreadyEnabled.into());
        }

        let setup = generate_setup(policy, &user.email)?;
        let mut active_user: users::ActiveModel = user.into();
        active_user.two_factor_auth_pending_secret_base32 = Set(Some(setup.secret_base32));
        active_user.updated_at = Set(Utc::now().fixed_offset());
        active_user.update(db).await?;

        Ok(setup.dto)
    }

    pub async fn enable_totp(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        policy: &TotpPolicy,
        dto: VerifyTotpCodeDto,
    ) -> Result<TotpEnableDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        if user.two_factor_auth_enabled {
            return Err(Totp2FaError::AlreadyEnabled.into());
        }

        let pending_secret = user
            .two_factor_auth_pending_secret_base32
            .clone()
            .ok_or(Totp2FaError::SetupNotStarted)?;

        let verification = verify_code(policy, &user.email, &pending_secret, &dto.code, None)?;
        let backup_codes = generate_backup_codes();
        let backup_code_hashes = hash_backup_codes(&backup_codes)?;
        let backup_code_hashes_json = Self::serialize_backup_code_hashes(&backup_code_hashes)?;
        let backup_codes_file = backup_codes_file_content(&backup_codes);

        let mut active_user: users::ActiveModel = user.into();
        active_user.two_factor_auth_enabled = Set(true);
        active_user.two_factor_auth_secret_base32 = Set(Some(pending_secret));
        active_user.two_factor_auth_pending_secret_base32 = Set(None);
        active_user.two_factor_auth_last_used_step = Set(Some(verification.new_last_used_step));
        active_user.two_factor_auth_backup_code_hashes = Set(Some(backup_code_hashes_json));
        active_user.updated_at = Set(Utc::now().fixed_offset());

        let updated = active_user.update(db).await?;
        Ok(TotpEnableDto {
            status: Self::to_totp_status(&updated),
            backup_codes,
            backup_codes_file,
        })
    }

    pub async fn cancel_totp_setup(db: &impl ConnectionTrait, user_id: Uuid) -> Result<TotpStatusDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;

        let mut active_user: users::ActiveModel = user.into();
        active_user.two_factor_auth_pending_secret_base32 = Set(None);
        active_user.updated_at = Set(Utc::now().fixed_offset());

        let updated = active_user.update(db).await?;
        Ok(Self::to_totp_status(&updated))
    }

    pub async fn disable_totp(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        policy: &TotpPolicy,
        dto: DisableTotpDto,
    ) -> Result<TotpStatusDto, AppError> {
        let user = Self::get_user_model(db, user_id).await?;
        if !user.two_factor_auth_enabled {
            return Err(Totp2FaError::NotEnabled.into());
        }

        compare_password(dto.password, user.password_hash.clone())?;

        let mut active_user: users::ActiveModel = user.clone().into();
        Self::apply_second_factor_verification(
            &user,
            &mut active_user,
            policy,
            dto.two_factor_auth_code.as_deref(),
            dto.two_factor_auth_backup_code.as_deref(),
        )?;

        active_user.two_factor_auth_enabled = Set(false);
        active_user.two_factor_auth_secret_base32 = Set(None);
        active_user.two_factor_auth_pending_secret_base32 = Set(None);
        active_user.two_factor_auth_backup_code_hashes = Set(None);
        active_user.updated_at = Set(Utc::now().fixed_offset());

        let updated = active_user.update(db).await?;
        Ok(Self::to_totp_status(&updated))
    }

    pub async fn verify_totp_for_login(
        db: &impl ConnectionTrait,
        user: UserModel,
        policy: &TotpPolicy,
        code: Option<String>,
        backup_code: Option<String>,
    ) -> Result<UserModel, AppError> {
        if !user.two_factor_auth_enabled {
            return Ok(user);
        }

        let mut active_user: users::ActiveModel = user.clone().into();
        Self::apply_second_factor_verification(
            &user,
            &mut active_user,
            policy,
            code.as_deref(),
            backup_code.as_deref(),
        )?;
        active_user.updated_at = Set(Utc::now().fixed_offset());

        Ok(active_user.update(db).await?)
    }

    fn apply_second_factor_verification(
        user: &UserModel,
        active_user: &mut users::ActiveModel,
        policy: &TotpPolicy,
        code: Option<&str>,
        backup_code: Option<&str>,
    ) -> Result<(), AppError> {
        if code.is_some() && backup_code.is_some() {
            return Err(Totp2FaError::InvalidCode.into());
        }

        if let Some(backup_code) = backup_code {
            let stored_hashes = Self::backup_code_hashes_from_json(user.two_factor_auth_backup_code_hashes.as_ref());
            let verification = verify_backup_code(backup_code, &stored_hashes)?;
            let remaining_hashes_json = Self::serialize_backup_code_hashes(&verification.remaining_hashes)?;
            active_user.two_factor_auth_backup_code_hashes = Set(Some(remaining_hashes_json));
            return Ok(());
        }

        let code = code.ok_or(Totp2FaError::CodeRequired)?;
        let secret = user
            .two_factor_auth_secret_base32
            .clone()
            .ok_or(Totp2FaError::InvalidSecret)?;
        let verification = verify_code(policy, &user.email, &secret, code, user.two_factor_auth_last_used_step)?;
        active_user.two_factor_auth_last_used_step = Set(Some(verification.new_last_used_step));

        Ok(())
    }

    fn to_totp_status(user: &UserModel) -> TotpStatusDto {
        let backup_codes_remaining =
            Self::backup_code_hashes_from_json(user.two_factor_auth_backup_code_hashes.as_ref()).len();
        TotpStatusDto {
            enabled: user.two_factor_auth_enabled,
            setup_pending: user.two_factor_auth_pending_secret_base32.is_some(),
            backup_codes_remaining,
        }
    }
}
