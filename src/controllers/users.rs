use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, patch},
};
use serde_json::json;

use crate::{
    dtos::user::{
        UpdateNotificationPreferencesDto, UpdateUserPasswordDto, UpdateUserPreferencesDto, UpdateUserProfileDto,
        UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::users::UserService,
    state::AppState,
};

async fn get_me(user: UserDto, service: UserService) -> Result<impl IntoResponse, AppError> {
    let me = service.get_me(user.id).await?;
    Ok(Json(me))
}

async fn update_profile(
    State(state): State<AppState>,
    user: UserDto,
    service: UserService,
    ValidatedJson(payload): ValidatedJson<UpdateUserProfileDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = service.update_profile(user.id, payload).await?;

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

async fn update_password(
    user: UserDto,
    service: UserService,
    ValidatedJson(payload): ValidatedJson<UpdateUserPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    service.update_password(user.id, payload).await?;

    Ok(Json(json!({
        "message": "Contraseña actualizada correctamente"
    })))
}

async fn update_preferences(
    user: UserDto,
    service: UserService,
    ValidatedJson(payload): ValidatedJson<UpdateUserPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = service.update_preferences(user.id, payload).await?;
    Ok(Json(updated))
}

async fn update_notifications(
    user: UserDto,
    service: UserService,
    ValidatedJson(payload): ValidatedJson<UpdateNotificationPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = service.update_notifications(user.id, payload).await?;
    Ok(Json(updated))
}

pub fn users_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/profile", patch(update_profile))
        .route("/me/password", patch(update_password))
        .route("/me/preferences", patch(update_preferences))
        .route("/me/preferences/notifications", patch(update_notifications))
}
