use axum::{
    Json, Router,
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        client::ClientAuthDto,
        milestone::{MilestoneDto, client_milestone_filter_schema, milestone_filter_schema},
        query::QueryPayload,
        user::UserDto,
    },
    error::AppError,
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::milestones::MilestoneService,
    state::AppState,
};

async fn search_milestones(
    user: UserDto,
    service: MilestoneService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> Result<impl IntoResponse, AppError> {
    milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) = service.search(user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

async fn client_search_milestones(
    client: ClientAuthDto,
    service: MilestoneService,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    client_milestone_filter_schema().validate_payload(&payload)?;

    let (milestones, total_items) = service
        .client_search_project_milestones(client.user_id, client.client_id, id, payload.clone())
        .await?;

    Ok(Json(json!({
        "payload": payload,
        "items": milestones,
        "total": total_items
    })))
}

async fn create_milestone(
    user: UserDto,
    service: MilestoneService,
    ValidatedJson(payload): ValidatedJson<MilestoneDto>,
) -> Result<impl IntoResponse, AppError> {
    let milestone = service.create(user.id, payload).await?;
    Ok(Json(milestone))
}

async fn update_milestone(
    user: UserDto,
    service: MilestoneService,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<MilestoneDto>,
) -> Result<impl IntoResponse, AppError> {
    let milestone = service.update(user.id, id, payload).await?;
    Ok(Json(milestone))
}

async fn delete_milestone(
    user: UserDto,
    service: MilestoneService,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    service.delete(user.id, id).await?;
    Ok(Json(json!({ "message": "Hito eliminado" })))
}

pub fn milestones_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_milestones))
        .route("/", post(create_milestone))
        .route("/client/search-by-project/{id}", get(client_search_milestones))
        .route("/{id}", axum::routing::put(update_milestone).delete(delete_milestone))
}
