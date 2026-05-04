use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        query::QueryPayload,
        time_entries::{
            CreateTimeEntryDto, StartTimeEntryTimerDto, StopTimeEntryTimerDto, UpdateTimeEntryDto,
            time_entries_filter_schema,
        },
        user::UserDto,
        utils::AppResult,
    },
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::time_entries::TimeEntryService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/time-entries/search",
    tag = "Time Entries",
    params(QueryPayload),
    responses(
        (status = 200, description = "Paginated list of time entries"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_time_entries(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> AppResult<impl IntoResponse> {
    time_entries_filter_schema().validate_payload(&payload)?;

    let (items, total_items, daily_totals_by_date, daily_limit_minutes) =
        TimeEntryService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total_items,
        "daily_totals_by_date": daily_totals_by_date,
        "daily_limit_minutes": daily_limit_minutes
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/time-entries/{id}",
    tag = "Time Entries",
    params(("id" = Uuid, Path, description = "Time entry id")),
    responses(
        (status = 200, description = "Time entry detail", body = crate::dtos::time_entries::TimeEntryDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Time entry not found")
    )
)]
pub(crate) async fn get_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::get_by_id(state.db.as_ref(), user.id, id).await?;

    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/v1/time-entries/",
    tag = "Time Entries",
    request_body = CreateTimeEntryDto,
    responses(
        (status = 200, description = "Time entry created", body = crate::dtos::time_entries::TimeEntryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_manual_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateTimeEntryDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::create_manual(state.db.as_ref(), user.id, payload).await?;

    Ok(Json(entry))
}

#[utoipa::path(
    put,
    path = "/api/v1/time-entries/{id}",
    tag = "Time Entries",
    params(("id" = Uuid, Path, description = "Time entry id")),
    request_body = UpdateTimeEntryDto,
    responses(
        (status = 200, description = "Time entry updated", body = crate::dtos::time_entries::TimeEntryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Time entry not found")
    )
)]
pub(crate) async fn update_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTimeEntryDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::update(state.db.as_ref(), user.id, id, payload).await?;

    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/v1/time-entries/timer/start",
    tag = "Time Entries",
    request_body = StartTimeEntryTimerDto,
    responses(
        (status = 200, description = "Timer started", body = crate::dtos::time_entries::TimeEntryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn start_time_entry_timer(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<StartTimeEntryTimerDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::start_timer(state.db.as_ref(), user.id, payload).await?;

    Ok(Json(entry))
}

#[utoipa::path(
    patch,
    path = "/api/v1/time-entries/{id}/timer/stop",
    tag = "Time Entries",
    params(("id" = Uuid, Path, description = "Time entry id")),
    request_body = StopTimeEntryTimerDto,
    responses(
        (status = 200, description = "Timer stopped", body = crate::dtos::time_entries::TimeEntryDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Time entry not found")
    )
)]
pub(crate) async fn stop_time_entry_timer(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<StopTimeEntryTimerDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::stop_timer(state.db.as_ref(), user.id, id, payload).await?;

    Ok(Json(entry))
}

#[utoipa::path(
    get,
    path = "/api/v1/time-entries/timer/active",
    tag = "Time Entries",
    responses(
        (status = 200, description = "Current active timer"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn get_active_time_entry_timer(
    State(state): State<AppState>,
    user: UserDto,
) -> AppResult<impl IntoResponse> {
    let active = TimeEntryService::get_active_timer(state.db.as_ref(), user.id).await?;

    Ok(Json(json!({ "item": active })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/time-entries/{id}",
    tag = "Time Entries",
    params(("id" = Uuid, Path, description = "Time entry id")),
    responses(
        (status = 200, description = "Time entry deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Time entry not found")
    )
)]
pub(crate) async fn delete_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    TimeEntryService::delete(state.db.as_ref(), user.id, id).await?;

    Ok(Json(json!({ "message": "Registro de tiempo eliminado" })))
}

pub fn time_entries_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_time_entries))
        .route("/", post(create_manual_time_entry))
        .route("/timer/active", get(get_active_time_entry_timer))
        .route("/timer/start", post(start_time_entry_timer))
        .route(
            "/{id}",
            get(get_time_entry).put(update_time_entry).delete(delete_time_entry),
        )
        .route("/{id}/timer/stop", patch(stop_time_entry_timer))
}
