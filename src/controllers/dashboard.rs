use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};

use crate::{
    dtos::{
        client::ClientAuthDto,
        dashboard::{
            AccountsReceivableFiltersDto, BillableRatioFiltersDto, ClientChangeRequestStatusFiltersDto,
            ClientMilestoneHealthFiltersDto, ClientProjectPortfolioFiltersDto, ClientProjectStatusFiltersDto,
            MonthlyRevenueFiltersDto, TaskBottlenecksFiltersDto, TimeByActivityFiltersDto,
        },
        user::UserDto,
        utils::AppResult,
    },
    services::dashboard::DashboardService,
    state::AppState,
};

async fn get_dashboard(State(state): State<AppState>, user: UserDto) -> AppResult<impl IntoResponse> {
    let overview = DashboardService::get_overview(state.db.as_ref(), user.id).await?;
    Ok(Json(overview))
}

async fn get_client_dashboard(State(state): State<AppState>, client: ClientAuthDto) -> AppResult<impl IntoResponse> {
    let overview = DashboardService::get_client_overview(state.db.as_ref(), client.user_id, client.client_id).await?;
    Ok(Json(overview))
}

async fn get_monthly_revenue(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<MonthlyRevenueFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_monthly_revenue(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

async fn get_accounts_receivable(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<AccountsReceivableFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_accounts_receivable(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

async fn get_time_by_activity(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<TimeByActivityFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_time_by_activity(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

async fn get_billable_ratio(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<BillableRatioFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_billable_ratio(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

async fn get_task_bottlenecks(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<TaskBottlenecksFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_task_bottlenecks(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

async fn get_client_project_status(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientProjectStatusFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows =
        DashboardService::query_client_project_status(state.db.as_ref(), client.user_id, client.client_id, &filters)
            .await?;
    Ok(Json(rows))
}

async fn get_client_milestone_health(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientMilestoneHealthFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows =
        DashboardService::query_client_milestone_health(state.db.as_ref(), client.user_id, client.client_id, &filters)
            .await?;
    Ok(Json(rows))
}

async fn get_client_change_request_status(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientChangeRequestStatusFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows = DashboardService::query_client_change_request_status(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        &filters,
    )
    .await?;
    Ok(Json(rows))
}

async fn get_client_project_portfolio(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientProjectPortfolioFiltersDto>,
) -> AppResult<impl IntoResponse> {
    let rows =
        DashboardService::query_client_project_portfolio(state.db.as_ref(), client.user_id, client.client_id, &filters)
            .await?;
    Ok(Json(rows))
}

fn group_charts() -> Router<AppState> {
    Router::new()
        .route("/monthly-revenue", get(get_monthly_revenue))
        .route("/accounts-receivable", get(get_accounts_receivable))
        .route("/time-by-activity", get(get_time_by_activity))
        .route("/billable-ratio", get(get_billable_ratio))
        .route("/task-bottlenecks", get(get_task_bottlenecks))
}

fn group_client_charts() -> Router<AppState> {
    Router::new()
        .route("/project-status", get(get_client_project_status))
        .route("/milestone-health", get(get_client_milestone_health))
        .route("/change-request-status", get(get_client_change_request_status))
        .route("/project-portfolio", get(get_client_project_portfolio))
}

pub fn dashboard_controller() -> Router<AppState> {
    Router::new()
        .route("/", get(get_dashboard))
        .nest("/charts", group_charts())
        .route("/client", get(get_client_dashboard))
        .nest("/client/charts", group_client_charts())
}
