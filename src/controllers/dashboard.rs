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
    middlewares::rate_limit::{CHARTS_RATE_LIMIT, check_user_rate_limit},
    services::dashboard::DashboardService,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Freelancer dashboard overview", body = crate::dtos::dashboard::DashboardDto),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_dashboard(State(state): State<AppState>, user: UserDto) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let overview = DashboardService::get_overview(state.db.as_ref(), user.id).await?;
    Ok(Json(overview))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/client",
    tag = "Dashboard",
    responses(
        (status = 200, description = "Client portal dashboard overview", body = crate::dtos::dashboard::ClientDashboardDto),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_client_dashboard(
    State(state): State<AppState>,
    client: ClientAuthDto,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, client.client_id, CHARTS_RATE_LIMIT)?;
    let overview = DashboardService::get_client_overview(state.db.as_ref(), client.user_id, client.client_id).await?;
    Ok(Json(overview))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/charts/monthly-revenue",
    tag = "Dashboard",
    params(MonthlyRevenueFiltersDto),
    responses(
        (status = 200, description = "Monthly revenue chart rows", body = [crate::dtos::dashboard::MonthlyRevenueRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_monthly_revenue(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<MonthlyRevenueFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_monthly_revenue(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/charts/accounts-receivable",
    tag = "Dashboard",
    params(AccountsReceivableFiltersDto),
    responses(
        (status = 200, description = "Accounts receivable rows", body = [crate::dtos::dashboard::AccountsReceivableRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_accounts_receivable(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<AccountsReceivableFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_accounts_receivable(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/charts/time-by-activity",
    tag = "Dashboard",
    params(TimeByActivityFiltersDto),
    responses(
        (status = 200, description = "Time-by-activity rows", body = [crate::dtos::dashboard::TimeByActivityRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_time_by_activity(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<TimeByActivityFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_time_by_activity(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/charts/billable-ratio",
    tag = "Dashboard",
    params(BillableRatioFiltersDto),
    responses(
        (status = 200, description = "Billable ratio daily rows", body = [crate::dtos::dashboard::BillableRatioDailyRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_billable_ratio(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<BillableRatioFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_billable_ratio(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/charts/task-bottlenecks",
    tag = "Dashboard",
    params(TaskBottlenecksFiltersDto),
    responses(
        (status = 200, description = "Task bottlenecks rows", body = [crate::dtos::dashboard::TaskBottleneckRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_task_bottlenecks(
    State(state): State<AppState>,
    user: UserDto,
    Query(filters): Query<TaskBottlenecksFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, user.id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_task_bottlenecks(state.db.as_ref(), user.id, &filters).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/client/charts/project-status",
    tag = "Dashboard",
    params(ClientProjectStatusFiltersDto),
    responses(
        (status = 200, description = "Client project status rows", body = [crate::dtos::dashboard::ClientProjectStatusRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_client_project_status(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientProjectStatusFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, client.client_id, CHARTS_RATE_LIMIT)?;
    let rows =
        DashboardService::query_client_project_status(state.db.as_ref(), client.user_id, client.client_id, &filters)
            .await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/client/charts/milestone-health",
    tag = "Dashboard",
    params(ClientMilestoneHealthFiltersDto),
    responses(
        (status = 200, description = "Client milestone health rows", body = [crate::dtos::dashboard::ClientMilestoneHealthRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_client_milestone_health(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientMilestoneHealthFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, client.client_id, CHARTS_RATE_LIMIT)?;
    let rows =
        DashboardService::query_client_milestone_health(state.db.as_ref(), client.user_id, client.client_id, &filters)
            .await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/client/charts/change-request-status",
    tag = "Dashboard",
    params(ClientChangeRequestStatusFiltersDto),
    responses(
        (status = 200, description = "Client change request status rows", body = [crate::dtos::dashboard::ClientChangeRequestStatusRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_client_change_request_status(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientChangeRequestStatusFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, client.client_id, CHARTS_RATE_LIMIT)?;
    let rows = DashboardService::query_client_change_request_status(
        state.db.as_ref(),
        client.user_id,
        client.client_id,
        &filters,
    )
    .await?;
    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/client/charts/project-portfolio",
    tag = "Dashboard",
    params(ClientProjectPortfolioFiltersDto),
    responses(
        (status = 200, description = "Client project portfolio rows", body = [crate::dtos::dashboard::ClientProjectPortfolioRow]),
        (status = 401, description = "Unauthorized"),
        (status = 429, description = "Rate limited")
    )
)]
pub(crate) async fn get_client_project_portfolio(
    State(state): State<AppState>,
    client: ClientAuthDto,
    Query(filters): Query<ClientProjectPortfolioFiltersDto>,
) -> AppResult<impl IntoResponse> {
    check_user_rate_limit(&state.rate_limit_charts, client.client_id, CHARTS_RATE_LIMIT)?;
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
