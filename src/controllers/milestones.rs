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
        client::ClientAuthDto,
        milestone::{CreateMilestoneDto, UpdateMilestoneDto, client_milestone_filter_schema, milestone_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::milestones::MilestoneService,
    state::AppState,
};

async fn search_milestones(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) = MilestoneService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

async fn client_search_milestones(
    State(state): State<AppState>,
    client: ClientAuthDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    client_milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) =
        MilestoneService::client_search_project_milestones(state.db.as_ref(), client.user_id, client.client_id, id, payload.clone())
            .await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

async fn create_milestone(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateMilestoneDto>,
) -> Result<impl IntoResponse, AppError> {
    let milestone = MilestoneService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(milestone))
}

async fn update_milestone(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateMilestoneDto>,
) -> Result<impl IntoResponse, AppError> {
    let milestone = MilestoneService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(milestone))
}

async fn delete_milestone(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    MilestoneService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Hito eliminado" })))
}

pub fn milestones_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_milestones))
        .route("/", post(create_milestone))
        .route("/client/search-by-project/{id}", get(client_search_milestones))
        .route("/{id}", axum::routing::put(update_milestone).delete(delete_milestone))
}
