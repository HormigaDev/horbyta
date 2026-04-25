use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use uuid::Uuid;

use crate::{
    dtos::dashboard::{
        AccountsReceivableRow, BillableRatioDailyRow, CurrencyAmountDto, DashboardDto, DashboardKpisDto,
        MonthlyRevenueRow, ScopeCreepValueRow, TaskBottleneckRow, TimeByActivityRow,
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

pub struct DashboardService;

impl DashboardService {
    pub async fn get_overview(db: &impl ConnectionTrait, user_id: Uuid) -> Result<DashboardDto, AppError> {
        let monthly_revenue = Self::query_view::<MonthlyRevenueRow>(db, MONTHLY_REVENUE_SQL, user_id).await?;
        let accounts_receivable = Self::query_view::<AccountsReceivableRow>(db, ACCOUNTS_RECEIVABLE_SQL, user_id).await?;
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

    async fn query_view<T>(db: &impl ConnectionTrait, sql: &str, user_id: Uuid) -> Result<Vec<T>, AppError>
    where
        T: FromQueryResult,
    {
        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, [Value::from(user_id)]);

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
