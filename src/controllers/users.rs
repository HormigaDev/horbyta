use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde_json::json;

use crate::{
    dtos::user::{
        PinTaskDto, UpdateNotificationPreferencesDto, UpdateUserPasswordDto, UpdateUserPreferencesDto,
        UpdateUserProfileDto, UserDto,
    },
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::users::UserService,
    state::AppState,
};

async fn get_me(State(state): State<AppState>, user: UserDto) -> Result<impl IntoResponse, AppError> {
    let me = UserService::get_me(state.db.as_ref(), user.id).await?;
    Ok(Json(me))
}

async fn update_profile(
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

async fn update_password(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    UserService::update_password(state.db.as_ref(), user.id, payload).await?;

    Ok(Json(json!({
        "message": "Contraseña actualizada correctamente"
    })))
}

async fn update_preferences(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateUserPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_preferences(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(updated))
}

async fn update_notifications(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<UpdateNotificationPreferencesDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::update_notifications(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(updated))
}

async fn pin_task(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<PinTaskDto>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::pin_task(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(updated))
}

async fn unpin_task(
    State(state): State<AppState>,
    user: UserDto,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let updated = UserService::unpin_task(state.db.as_ref(), user.id, &task_id).await?;
    Ok(Json(updated))
}

pub fn users_controller() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/profile", patch(update_profile))
        .route("/me/password", patch(update_password))
        .route("/me/preferences", patch(update_preferences))
        .route("/me/preferences/notifications", patch(update_notifications))
        .route("/me/preferences/pinned-tasks", post(pin_task))
        .route("/me/preferences/pinned-tasks/{task_id}", delete(unpin_task))
}
