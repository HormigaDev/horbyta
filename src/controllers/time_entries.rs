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

async fn search_time_entries(
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

async fn get_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::get_by_id(state.db.as_ref(), user.id, id).await?;

    Ok(Json(entry))
}

async fn create_manual_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateTimeEntryDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::create_manual(state.db.as_ref(), user.id, payload).await?;

    Ok(Json(entry))
}

async fn update_time_entry(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateTimeEntryDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::update(state.db.as_ref(), user.id, id, payload).await?;

    Ok(Json(entry))
}

async fn start_time_entry_timer(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<StartTimeEntryTimerDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::start_timer(state.db.as_ref(), user.id, payload).await?;

    Ok(Json(entry))
}

async fn stop_time_entry_timer(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<StopTimeEntryTimerDto>,
) -> AppResult<impl IntoResponse> {
    let entry = TimeEntryService::stop_timer(state.db.as_ref(), user.id, id, payload).await?;

    Ok(Json(entry))
}

async fn get_active_time_entry_timer(State(state): State<AppState>, user: UserDto) -> AppResult<impl IntoResponse> {
    let active = TimeEntryService::get_active_timer(state.db.as_ref(), user.id).await?;

    Ok(Json(json!({ "item": active })))
}

async fn delete_time_entry(
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
