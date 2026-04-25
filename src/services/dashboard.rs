use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use uuid::Uuid;

use crate::{
    dtos::dashboard::{
        AccountsReceivableRow, BillableRatioDailyRow, ClientApprovedChangeValueRow, ClientChangeRequestStatusRow,
        ClientDashboardDto, ClientDashboardKpisDto, ClientMilestoneHealthRow, ClientProjectPortfolioRow,
        ClientProjectStatusRow, CurrencyAmountDto, DashboardDto, DashboardKpisDto, MonthlyRevenueRow,
        ScopeCreepValueRow, TaskBottleneckRow, TimeByActivityRow,
    },
    error::AppError,
};

const MONTHLY_REVENUE_SQL: &str = r#"
    select currency_type, month_date, total_revenue
    from vw_dash_monthly_revenue
    where user_id = $1
    order by month_date asc
"#;

const ACCOUNTS_RECEIVABLE_SQL: &str = r#"
    select client_name, currency_type, outstanding_balance, oldest_due_date
    from vw_dash_accounts_receivable
    where user_id = $1
    order by oldest_due_date asc nulls last, client_name asc
"#;

const TIME_BY_ACTIVITY_SQL: &str = r#"
    select activity_name, billable, week_date, total_hours
    from vw_dash_time_by_activity
    where user_id = $1
    order by week_date asc, activity_name asc
"#;

const BILLABLE_RATIO_DAILY_SQL: &str = r#"
    select work_date, billable_hours, no_billable_hours, total_hours
    from vw_dash_billable_ratio_daily
    where user_id = $1
    order by work_date asc
"#;

const SCOPE_CREEP_VALUE_SQL: &str = r#"
    select currency_type, total_approved_changes, total_additional_revenue, total_days_extended
    from vw_dash_scope_creep_value
    where user_id = $1
    order by currency_type asc
"#;

const TASK_BOTTLENECKS_SQL: &str = r#"
    select project_id, project_name, total_tasks, tasks_todo, tasks_in_progress, tasks_in_review, overdue_tasks
    from vw_dash_task_bottlenecks
    where user_id = $1
    order by overdue_tasks desc, tasks_in_progress desc, project_name asc
"#;

const CLIENT_PROJECT_STATUS_SQL: &str = r#"
    select status, total_projects
    from vw_client_dash_project_status
    where user_id = $1 and client_id = $2
    order by total_projects desc, status asc
"#;

const CLIENT_MILESTONE_HEALTH_SQL: &str = r#"
    select project_id, project_name, total_milestones, completed_milestones, overdue_milestones, upcoming_milestones, next_due_date
    from vw_client_dash_milestone_health
    where user_id = $1 and client_id = $2
    order by overdue_milestones desc, upcoming_milestones desc, project_name asc
"#;

const CLIENT_CHANGE_REQUEST_STATUS_SQL: &str = r#"
    select status, total_requests
    from vw_client_dash_change_request_status
    where user_id = $1 and client_id = $2
    order by total_requests desc, status asc
"#;

const CLIENT_APPROVED_CHANGE_VALUE_SQL: &str = r#"
    select currency_type, total_approved_changes, total_approved_value
    from vw_client_dash_approved_change_value
    where user_id = $1 and client_id = $2
    order by currency_type asc
"#;

const CLIENT_PROJECT_PORTFOLIO_SQL: &str = r#"
    select
        project_id,
        project_name,
        project_status,
        billable,
        currency_type,
        billing_type,
        hourly_rate,
        fixed_price,
        retainer_amount,
        payment_day,
        total_milestones,
        completed_milestones,
        overdue_milestones,
        upcoming_milestones,
        next_milestone_name,
        next_milestone_due_date
    from vw_client_dash_project_portfolio
    where user_id = $1 and client_id = $2
    order by overdue_milestones desc, upcoming_milestones desc, project_name asc
"#;

pub struct DashboardService;

impl DashboardService {
    pub async fn get_overview(db: &impl ConnectionTrait, user_id: Uuid) -> Result<DashboardDto, AppError> {
        let monthly_revenue = Self::query_view::<MonthlyRevenueRow>(db, MONTHLY_REVENUE_SQL, user_id).await?;
        let accounts_receivable =
            Self::query_view::<AccountsReceivableRow>(db, ACCOUNTS_RECEIVABLE_SQL, user_id).await?;
        let time_by_activity = Self::query_view::<TimeByActivityRow>(db, TIME_BY_ACTIVITY_SQL, user_id).await?;
        let billable_ratio_daily =
            Self::query_view::<BillableRatioDailyRow>(db, BILLABLE_RATIO_DAILY_SQL, user_id).await?;
        let scope_creep_value = Self::query_view::<ScopeCreepValueRow>(db, SCOPE_CREEP_VALUE_SQL, user_id).await?;
        let task_bottlenecks = Self::query_view::<TaskBottleneckRow>(db, TASK_BOTTLENECKS_SQL, user_id).await?;

        let kpis = DashboardKpisDto {
            scope_creep_recovered: scope_creep_value
                .iter()
                .map(|row| CurrencyAmountDto {
                    currency_type: row.currency_type.clone(),
                    amount: row.total_additional_revenue,
                })
                .collect(),
            accounts_receivable_total: Self::aggregate_by_currency(&accounts_receivable),
            overdue_tasks_total: task_bottlenecks.iter().map(|row| row.overdue_tasks).sum(),
        };

        Ok(DashboardDto {
            kpis,
            monthly_revenue,
            accounts_receivable,
            time_by_activity,
            billable_ratio_daily,
            scope_creep_value,
            task_bottlenecks,
        })
    }

    pub async fn get_client_overview(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<ClientDashboardDto, AppError> {
        let project_status =
            Self::query_client_view::<ClientProjectStatusRow>(db, CLIENT_PROJECT_STATUS_SQL, user_id, client_id)
                .await?;
        let milestone_health =
            Self::query_client_view::<ClientMilestoneHealthRow>(db, CLIENT_MILESTONE_HEALTH_SQL, user_id, client_id)
                .await?;
        let change_request_status = Self::query_client_view::<ClientChangeRequestStatusRow>(
            db,
            CLIENT_CHANGE_REQUEST_STATUS_SQL,
            user_id,
            client_id,
        )
        .await?;
        let approved_change_value = Self::query_client_view::<ClientApprovedChangeValueRow>(
            db,
            CLIENT_APPROVED_CHANGE_VALUE_SQL,
            user_id,
            client_id,
        )
        .await?;
        let project_portfolio =
            Self::query_client_view::<ClientProjectPortfolioRow>(db, CLIENT_PROJECT_PORTFOLIO_SQL, user_id, client_id)
                .await?;

        let active_projects_total = project_status
            .iter()
            .find(|row| row.status.to_string() == "open")
            .map(|row| row.total_projects)
            .unwrap_or(0);

        let overdue_milestones_total = milestone_health.iter().map(|row| row.overdue_milestones).sum();
        let open_change_requests_total = change_request_status
            .iter()
            .filter(|row| {
                let status = row.status.to_string();
                status == "submitted" || status == "assessed" || status == "approved"
            })
            .map(|row| row.total_requests)
            .sum();

        let kpis = ClientDashboardKpisDto {
            active_projects_total,
            overdue_milestones_total,
            open_change_requests_total,
            approved_change_value: approved_change_value
                .iter()
                .map(|row| CurrencyAmountDto {
                    currency_type: row.currency_type.clone(),
                    amount: row.total_approved_value,
                })
                .collect(),
        };

        Ok(ClientDashboardDto {
            kpis,
            project_status,
            milestone_health,
            change_request_status,
            approved_change_value,
            project_portfolio,
        })
    }

    async fn query_view<T>(db: &impl ConnectionTrait, sql: &str, user_id: Uuid) -> Result<Vec<T>, AppError>
    where
        T: FromQueryResult,
    {
        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, [Value::from(user_id)]);

        T::find_by_statement(stmt).all(db).await.map_err(AppError::Database)
    }

    async fn query_client_view<T>(
        db: &impl ConnectionTrait,
        sql: &str,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<T>, AppError>
    where
        T: FromQueryResult,
    {
        let stmt =
            Statement::from_sql_and_values(DbBackend::Postgres, sql, [Value::from(user_id), Value::from(client_id)]);

        T::find_by_statement(stmt).all(db).await.map_err(AppError::Database)
    }

    fn aggregate_by_currency(rows: &[AccountsReceivableRow]) -> Vec<CurrencyAmountDto> {
        let mut totals: Vec<CurrencyAmountDto> = Vec::new();

        for row in rows {
            if let Some(entry) = totals.iter_mut().find(|c| c.currency_type == row.currency_type) {
                entry.amount += row.outstanding_balance;
            } else {
                totals.push(CurrencyAmountDto {
                    currency_type: row.currency_type.clone(),
                    amount: row.outstanding_balance,
                });
            }
        }

        totals
    }
}
