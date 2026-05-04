use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, patch},
};
use serde_json::json;

use crate::{
    controllers::ui_state::ui_state_controller,
    dtos::user::{
        UpdateNotificationPreferencesDto, UpdateUserPasswordDto, UpdateUserPreferencesDto, UpdateUserProfileDto,
        UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
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
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn update_password(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    UserService::update_password(state.db.as_ref(), user.id, payload).await?;

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

pub fn users_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/profile", patch(update_profile))
        .route("/me/password", patch(update_password))
        .route("/me/preferences", patch(update_preferences))
        .route("/me/preferences/notifications", patch(update_notifications))
        .nest("/me/ui-state", ui_state_controller())
}
