use chrono::NaiveDate;
use sea_orm::{FromQueryResult, prelude::Decimal};
use serde::Serialize;
use uuid::Uuid;

use super::{change_request::ChangeRequestStatus, project::ProjectStatus, project_billing::ProjectBillingType};

#[derive(Debug, Serialize, FromQueryResult)]
pub struct MonthlyRevenueRow {
    pub currency_type: String,
    pub month_date: NaiveDate,
    pub total_revenue: Decimal,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct AccountsReceivableRow {
    pub client_name: String,
    pub currency_type: String,
    pub outstanding_balance: Decimal,
    pub oldest_due_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TimeByActivityRow {
    pub activity_name: String,
    pub billable: bool,
    pub week_date: NaiveDate,
    pub total_hours: Decimal,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct BillableRatioDailyRow {
    pub work_date: NaiveDate,
    pub billable_hours: Decimal,
    pub no_billable_hours: Decimal,
    pub total_hours: Decimal,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ScopeCreepValueRow {
    pub currency_type: String,
    pub total_approved_changes: i64,
    pub total_additional_revenue: Decimal,
    pub total_days_extended: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct TaskBottleneckRow {
    pub project_id: Uuid,
    pub project_name: String,
    pub total_tasks: i64,
    pub tasks_todo: i64,
    pub tasks_in_progress: i64,
    pub tasks_in_review: i64,
    pub overdue_tasks: i64,
}

#[derive(Debug, Serialize)]
pub struct CurrencyAmountDto {
    pub currency_type: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct DashboardKpisDto {
    pub scope_creep_recovered: Vec<CurrencyAmountDto>,
    pub accounts_receivable_total: Vec<CurrencyAmountDto>,
    pub overdue_tasks_total: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardDto {
    pub kpis: DashboardKpisDto,
    pub monthly_revenue: Vec<MonthlyRevenueRow>,
    pub accounts_receivable: Vec<AccountsReceivableRow>,
    pub time_by_activity: Vec<TimeByActivityRow>,
    pub billable_ratio_daily: Vec<BillableRatioDailyRow>,
    pub scope_creep_value: Vec<ScopeCreepValueRow>,
    pub task_bottlenecks: Vec<TaskBottleneckRow>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientProjectStatusRow {
    pub status: ProjectStatus,
    pub total_projects: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientMilestoneHealthRow {
    pub project_id: Uuid,
    pub project_name: String,
    pub total_milestones: i64,
    pub completed_milestones: i64,
    pub overdue_milestones: i64,
    pub upcoming_milestones: i64,
    pub next_due_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientChangeRequestStatusRow {
    pub status: ChangeRequestStatus,
    pub total_requests: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientApprovedChangeValueRow {
    pub currency_type: String,
    pub total_approved_changes: i64,
    pub total_approved_value: Decimal,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ClientProjectPortfolioRow {
    pub project_id: Uuid,
    pub project_name: String,
    pub project_status: ProjectStatus,
    pub billable: bool,
    pub currency_type: Option<String>,
    pub billing_type: Option<ProjectBillingType>,
    pub hourly_rate: Option<Decimal>,
    pub fixed_price: Option<Decimal>,
    pub retainer_amount: Option<Decimal>,
    pub payment_day: Option<i16>,
    pub total_milestones: i64,
    pub completed_milestones: i64,
    pub overdue_milestones: i64,
    pub upcoming_milestones: i64,
    pub next_milestone_name: Option<String>,
    pub next_milestone_due_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct ClientDashboardKpisDto {
    pub active_projects_total: i64,
    pub overdue_milestones_total: i64,
    pub open_change_requests_total: i64,
    pub approved_change_value: Vec<CurrencyAmountDto>,
}

#[derive(Debug, Serialize)]
pub struct ClientDashboardDto {
    pub kpis: ClientDashboardKpisDto,
    pub project_status: Vec<ClientProjectStatusRow>,
    pub milestone_health: Vec<ClientMilestoneHealthRow>,
    pub change_request_status: Vec<ClientChangeRequestStatusRow>,
    pub approved_change_value: Vec<ClientApprovedChangeValueRow>,
    pub project_portfolio: Vec<ClientProjectPortfolioRow>,
}
