use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;

use crate::{
    auth::totp_2fa::default_totp_policy,
    controllers::ui_state::ui_state_controller,
    dtos::totp_2fa::{DisableTotpDto, VerifyTotpCodeDto},
    dtos::user::{
        UpdateNotificationPreferencesDto, UpdateUserPasswordDto, UpdateUserPreferencesDto, UpdateUserProfileDto,
        UserDto,
    },
    error::{AppError, Totp2FaError},
    middlewares::{
        rate_limit::{TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT, check_key_rate_limit, record_key_rate_limit_failure},
        validator::ValidatedJson,
    },
    services::users::UserService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Current user settings", body = crate::dtos::user::UserSettingsDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn get_me(State(state): State<AppState>, user: UserDto) -> Result<impl IntoResponse, AppError> {
    let me = UserService::get_me(state.db.as_ref(), user.id).await?;
    Ok(Json(me))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me/profile",
    tag = "Users",
    request_body = UpdateUserProfileDto,
    responses(
        (status = 200, description = "User profile updated", body = crate::dtos::user::UserSettingsDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn update_profile(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserProfileDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_profile(state.db.as_ref(), user.id, payload).await?;

    state
        .user_cache
        .insert(
            updated.id,
            UserDto {
                id: updated.id,
                name: updated.name.clone(),
                email: updated.email.clone(),
                created_at: updated.created_at,
                updated_at: updated.updated_at,
            },
        )
        .await;

    Ok(Json(updated))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me/password",
    tag = "Users",
    request_body = UpdateUserPasswordDto,
    responses(
        (status = 200, description = "Password updated"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Too many two-factor authentication attempts")
    )
)]
pub(crate) async fn update_password(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    let two_factor_rate_limit_key = two_factor_rate_limit_key(user.id);
    check_key_rate_limit(
        &state.rate_limit_totp_failures,
        &two_factor_rate_limit_key,
        TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT,
    )?;

    let result = UserService::update_password(state.db.as_ref(), user.id, &default_totp_policy(), payload).await;
    record_two_factor_failure(&state, &two_factor_rate_limit_key, &result);
    result?;

    Ok(Json(json!({
        "message": "Password updated successfully"
    })))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me/preferences",
    tag = "Users",
    request_body = UpdateUserPreferencesDto,
    responses(
        (status = 200, description = "User preferences updated", body = crate::dtos::user::UserSettingsDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn update_preferences(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_preferences(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/me/preferences/notifications",
    tag = "Users",
    request_body = UpdateNotificationPreferencesDto,
    responses(
        (status = 200, description = "Notification preferences updated", body = crate::dtos::user::UserSettingsDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn update_notifications(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateNotificationPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_notifications(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me/2fa/status",
    tag = "Users",
    responses(
        (status = 200, description = "Two-factor authentication status", body = crate::dtos::totp_2fa::TotpStatusDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn get_totp_status(
    State(state): State<AppState>,
    user: UserDto,
) -> Result<impl IntoResponse, AppError> {
    let status = UserService::get_totp_status(state.db.as_ref(), user.id).await?;
    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/me/2fa/setup",
    tag = "Users",
    responses(
        (status = 200, description = "Two-factor authentication setup created", body = crate::dtos::totp_2fa::TotpSetupDto),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Two-factor authentication already enabled")
    )
)]
pub(crate) async fn start_totp_setup(
    State(state): State<AppState>,
    user: UserDto,
) -> Result<impl IntoResponse, AppError> {
    let setup = UserService::start_totp_setup(state.db.as_ref(), user.id, &default_totp_policy()).await?;
    Ok(Json(setup))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/me/2fa/enable",
    tag = "Users",
    request_body = VerifyTotpCodeDto,
    responses(
        (status = 200, description = "Two-factor authentication enabled", body = crate::dtos::totp_2fa::TotpEnableDto),
        (status = 400, description = "Setup not started or validation failed"),
        (status = 401, description = "Invalid two-factor authentication code"),
        (status = 409, description = "Two-factor authentication already enabled"),
        (status = 429, description = "Too many two-factor authentication attempts")
    )
)]
pub(crate) async fn enable_totp(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<VerifyTotpCodeDto>,
) -> Result<impl IntoResponse, AppError> {
    let two_factor_rate_limit_key = two_factor_rate_limit_key(user.id);
    check_key_rate_limit(
        &state.rate_limit_totp_failures,
        &two_factor_rate_limit_key,
        TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT,
    )?;

    let result = UserService::enable_totp(state.db.as_ref(), user.id, &default_totp_policy(), payload).await;
    record_two_factor_failure(&state, &two_factor_rate_limit_key, &result);
    let enabled = result?;
    Ok(Json(enabled))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/me/2fa/setup",
    tag = "Users",
    responses(
        (status = 200, description = "Two-factor authentication setup cancelled", body = crate::dtos::totp_2fa::TotpStatusDto),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn cancel_totp_setup(
    State(state): State<AppState>,
    user: UserDto,
) -> Result<impl IntoResponse, AppError> {
    let status = UserService::cancel_totp_setup(state.db.as_ref(), user.id).await?;
    Ok(Json(status))
}

#[utoipa::path(
    post,
    path = "/api/v1/users/me/2fa/disable",
    tag = "Users",
    request_body = DisableTotpDto,
    responses(
        (status = 200, description = "Two-factor authentication disabled", body = crate::dtos::totp_2fa::TotpStatusDto),
        (status = 400, description = "Two-factor authentication is not enabled or validation failed"),
        (status = 401, description = "Invalid password or two-factor authentication code"),
        (status = 429, description = "Too many two-factor authentication attempts")
    )
)]
pub(crate) async fn disable_totp(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<DisableTotpDto>,
) -> Result<impl IntoResponse, AppError> {
    let two_factor_rate_limit_key = two_factor_rate_limit_key(user.id);
    check_key_rate_limit(
        &state.rate_limit_totp_failures,
        &two_factor_rate_limit_key,
        TWO_FACTOR_AUTH_FAILURE_RATE_LIMIT,
    )?;

    let result = UserService::disable_totp(state.db.as_ref(), user.id, &default_totp_policy(), payload).await;
    record_two_factor_failure(&state, &two_factor_rate_limit_key, &result);
    let status = result?;
    Ok(Json(status))
}

pub fn users_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/profile", patch(update_profile))
        .route("/me/password", patch(update_password))
        .route("/me/preferences", patch(update_preferences))
        .route("/me/preferences/notifications", patch(update_notifications))
        .route("/me/2fa/status", get(get_totp_status))
        .route("/me/2fa/setup", post(start_totp_setup).delete(cancel_totp_setup))
        .route("/me/2fa/enable", post(enable_totp))
        .route("/me/2fa/disable", post(disable_totp))
        .nest("/me/ui-state", ui_state_controller())
}

fn two_factor_rate_limit_key(user_id: uuid::Uuid) -> String {
    format!("user-2fa:{user_id}")
}

fn record_two_factor_failure<T>(state: &AppState, key: &str, result: &Result<T, AppError>) {
    if matches!(
        result,
        Err(AppError::Totp2Fa(
            Totp2FaError::InvalidCode | Totp2FaError::ReplayedCode
        ))
    ) {
        record_key_rate_limit_failure(&state.rate_limit_totp_failures, key.to_string());
    }
}
