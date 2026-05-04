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

#[utoipa::path(
    get,
    path = "/api/v1/activities/search",
    tag = "Activities",
    params(QueryPayload),
    responses(
        (status = 200, description = "Paginated list of activities", body = serde_json::Value),
        (status = 400, description = "Invalid search payload"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_activities(
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

#[utoipa::path(
    get,
    path = "/api/v1/activities/{id}",
    tag = "Activities",
    params(("id" = Uuid, Path, description = "Activity id")),
    responses(
        (status = 200, description = "Activity detail", body = crate::dtos::activity::ActivityDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Activity not found")
    )
)]
pub(crate) async fn get_activity(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::get_by_id(state.db.as_ref(), user.id, id).await?;

    Ok(Json(activity))
}

#[utoipa::path(
    post,
    path = "/api/v1/activities/",
    tag = "Activities",
    request_body = CreateActivityDto,
    responses(
        (status = 200, description = "Activity created", body = crate::dtos::activity::ActivityDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_activity(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateActivityDto>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(activity))
}

#[utoipa::path(
    put,
    path = "/api/v1/activities/{id}",
    tag = "Activities",
    params(("id" = Uuid, Path, description = "Activity id")),
    request_body = UpdateActivityDto,
    responses(
        (status = 200, description = "Activity updated", body = crate::dtos::activity::ActivityDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Activity not found")
    )
)]
pub(crate) async fn update_activity(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateActivityDto>,
) -> AppResult<impl IntoResponse> {
    let activity = ActivityService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(activity))
}

#[utoipa::path(
    delete,
    path = "/api/v1/activities/{id}",
    tag = "Activities",
    params(("id" = Uuid, Path, description = "Activity id")),
    responses(
        (status = 200, description = "Activity deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Activity not found")
    )
)]
pub(crate) async fn delete_activity(
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
