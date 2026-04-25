use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    dtos::{client::ClientAuthDto, user::UserDto, utils::AppResult},
    services::dashboard::DashboardService,
    state::AppState,
};

pub async fn get_dashboard(State(state): State<AppState>, user: UserDto) -> AppResult<impl IntoResponse> {
    let overview = DashboardService::get_overview(state.db.as_ref(), user.id).await?;
    Ok(Json(overview))
}

pub async fn get_client_dashboard(
    State(state): State<AppState>,
    client: ClientAuthDto,
) -> AppResult<impl IntoResponse> {
    let overview = DashboardService::get_client_overview(state.db.as_ref(), client.user_id, client.client_id).await?;
    Ok(Json(overview))
}
