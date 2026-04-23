use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        activity::{CreateActivityDto, UpdateActivityDto, activity_filter_schema},
        query::QueryPayload,
        user::UserDto,
        utils::AppResult,
    },
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::activities::ActivityService,
    state::AppState,
};

async fn search_activities(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> AppResult<impl IntoResponse> {
    activity_filter_schema().validate_payload(&payload)?;

    let (activities, total_items) = ActivityService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": activities,
        "total": total_items
    })))
}

async fn get_activity(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::get_by_id(state.db.as_ref(), user.id, id).await?;

    Ok(Json(activity))
}

async fn create_activity(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateActivityDto>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(activity))
}

async fn update_activity(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateActivityDto>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(activity))
}

async fn delete_activity(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    ActivityService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({"message": "activity deleted successfully"})))
}

pub fn activities_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_activities))
        .route("/", post(create_activity))
        .route("/{id}", get(get_activity).put(update_activity).delete(delete_activity))
}
