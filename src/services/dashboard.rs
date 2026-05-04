use sea_orm::{ConnectionTrait, DbBackend, FromQueryResult, Statement, Value};
use uuid::Uuid;

use crate::{
    dtos::dashboard::{
        AccountsReceivableFiltersDto, AccountsReceivableRow, BillableRatioDailyRow, BillableRatioFiltersDto,
        ClientApprovedChangeValueRow, ClientChangeRequestStatusFiltersDto, ClientChangeRequestStatusRow,
        ClientDashboardDto, ClientDashboardKpisDto, ClientMilestoneHealthFiltersDto, ClientMilestoneHealthRow,
        ClientProjectPortfolioFiltersDto, ClientProjectPortfolioRow, ClientProjectStatusFiltersDto,
        ClientProjectStatusRow, CurrencyAmountDto, DashboardDto, DashboardKpisDto, MonthlyRevenueFiltersDto,
        MonthlyRevenueRow, ScopeCreepValueRow, TaskBottleneckRow, TaskBottlenecksFiltersDto, TimeByActivityFiltersDto,
        TimeByActivityRow,
    },
    error::AppError,
};

const MONTHLY_REVENUE_SQL: &str = r#"
    select
        p.currency_type,
        date_trunc('month', p.payment_date)::date as month_date,
        sum(case when p.transaction_type = 'charge' then p.amount else -p.amount end) as total_revenue
    from payments p
    inner join projects prj on p.project_id = prj.id
    where p.user_id = $1
      and ($2::date is null or p.payment_date >= $2)
      and ($3::date is null or p.payment_date <= $3)
      and ($4::varchar is null or p.currency_type = $4)
      and ($5::uuid is null or prj.client_id = $5)
      and ($6::uuid is null or p.project_id = $6)
    group by p.currency_type, date_trunc('month', p.payment_date)
    order by month_date asc
"#;

const ACCOUNTS_RECEIVABLE_SQL: &str = r#"
    with invoice_balances as (
        select
            i.id as invoice_id,
            i.user_id,
            i.currency_type,
            i.due_date,
            prj.client_id,
            i.amount - coalesce((
                select sum(case when p.transaction_type = 'charge' then p.amount else -p.amount end)
                from payments p
                where p.invoice_id = i.id
            ), 0) as invoice_outstanding
        from invoices i
        inner join projects prj on i.project_id = prj.id
        where i.status = 'open'
          and i.user_id = $1
          and ($2::uuid is null or prj.client_id = $2)
          and ($3::varchar is null or i.currency_type = $3)
          and ($4::date is null or i.due_date >= $4)
          and ($5::date is null or i.due_date <= $5)
          and ($6::boolean is not true or (i.due_date is not null and i.due_date < current_date))
    )
    select
        c.name as client_name,
        ib.currency_type,
        sum(ib.invoice_outstanding) as outstanding_balance,
        min(ib.due_date) as oldest_due_date
    from invoice_balances ib
    inner join clients c on ib.client_id = c.id
    group by c.name, ib.currency_type
    having sum(ib.invoice_outstanding) > 0
       and ($7::numeric is null or sum(ib.invoice_outstanding) >= $7)
    order by oldest_due_date asc nulls last, client_name asc
"#;

const TIME_BY_ACTIVITY_SQL: &str = r#"
    select
        a.name as activity_name,
        te.billable,
        date_trunc('week', te.work_date)::date as week_date,
        round(sum(te.duration_minutes) / 60.0, 2) as total_hours
    from time_entries te
    inner join activities a on te.activity = a.id
    inner join tasks t on te.task_id = t.id
    left join projects prj on t.project_id = prj.id
    where te.user_id = $1
      and ($2::date is null or te.work_date >= $2)
      and ($3::date is null or te.work_date <= $3)
      and ($4::uuid is null or te.activity = $4)
      and ($5::boolean is null or te.billable = $5)
      and ($6::uuid is null or prj.client_id = $6)
      and ($7::uuid is null or t.project_id = $7)
    group by a.name, te.billable, date_trunc('week', te.work_date)
    order by week_date asc, a.name asc
"#;

const BILLABLE_RATIO_DAILY_SQL: &str = r#"
    select
        te.work_date,
        round(sum(case when te.billable then te.duration_minutes else 0 end) / 60.0, 2) as billable_hours,
        round(sum(case when not te.billable then te.duration_minutes else 0 end) / 60.0, 2) as no_billable_hours,
        round(sum(te.duration_minutes) / 60.0, 2) as total_hours
    from time_entries te
    inner join tasks t on te.task_id = t.id
    left join projects prj on t.project_id = prj.id
    where te.user_id = $1
      and ($2::date is null or te.work_date >= $2)
      and ($3::date is null or te.work_date <= $3)
      and ($4::uuid is null or prj.client_id = $4)
      and ($5::uuid is null or t.project_id = $5)
      and ($6::uuid is null or te.activity = $6)
    group by te.work_date
    order by te.work_date asc
"#;

const SCOPE_CREEP_VALUE_SQL: &str = r#"
    select currency_type, total_approved_changes, total_additional_revenue, total_days_extended
    from vw_dash_scope_creep_value
    where user_id = $1
    order by currency_type asc
"#;

const TASK_BOTTLENECKS_SQL: &str = r#"
    select
        p.id as project_id,
        p.name as project_name,
        count(t.id) as total_tasks,
        sum(case when t.status = 'todo' then 1 else 0 end) as tasks_todo,
        sum(case when t.status = 'in_progress' then 1 else 0 end) as tasks_in_progress,
        sum(case when t.status = 'review' then 1 else 0 end) as tasks_in_review,
        sum(case when t.due_date < now() and t.status != 'done' then 1 else 0 end) as overdue_tasks
    from tasks t
    inner join projects p on t.project_id = p.id
    where p.status = 'open'
      and t.user_id = $1
      and ($2::uuid is null or p.client_id = $2)
      and ($3::uuid is null or p.id = $3)
      and ($4::varchar is null or t.status = $4)
      and ($5::int is null or t.priority = $5)
      and ($6::boolean is not true or (t.due_date < now() and t.status != 'done'))
    group by p.id, p.name
    order by overdue_tasks desc, tasks_in_progress desc, project_name asc
"#;

const CLIENT_PROJECT_STATUS_SQL: &str = r#"
    select p.status, count(*) as total_projects
    from projects p
    where p.user_id = $1
      and p.client_id = $2
      and ($3::varchar is null or p.status = $3)
      and ($4::boolean is null or p.billable = $4)
      and ($5::uuid is null or p.category_id = $5)
    group by p.status
    order by total_projects desc, p.status asc
"#;

const CLIENT_MILESTONE_HEALTH_SQL: &str = r#"
    with filtered_milestones as (
        select m.id, m.status, m.due_date,
               p.id as project_id, p.name as project_name
        from milestones m
        inner join projects p on m.project_id = p.id
        where p.user_id = $1
          and p.client_id = $2
          and ($3::uuid is null or p.id = $3)
          and ($4::varchar is null or m.status = $4)
          and ($5::date is null or m.due_date::date >= $5)
          and ($6::date is null or m.due_date::date <= $6)
          and (
              $7::boolean is not true
              or (
                  m.status not in ('approved', 'released', 'cancelled')
                  and m.due_date is not null
                  and m.due_date < now()
              )
          )
          and (
              $8::boolean is not true
              or (
                  m.status not in ('approved', 'released', 'cancelled')
                  and m.due_date is not null
                  and m.due_date >= now()
                  and m.due_date < now() + interval '7 days'
              )
          )
    )
    select
        project_id,
        project_name,
        count(*) as total_milestones,
        sum(case when status in ('approved', 'released') then 1 else 0 end) as completed_milestones,
        sum(
            case when status not in ('approved', 'released', 'cancelled')
                 and due_date is not null and due_date < now()
                 then 1 else 0 end
        ) as overdue_milestones,
        sum(
            case when status not in ('approved', 'released', 'cancelled')
                 and due_date is not null
                 and due_date >= now()
                 and due_date < now() + interval '7 days'
                 then 1 else 0 end
        ) as upcoming_milestones,
        min(due_date::date) filter (
            where status not in ('approved', 'released', 'cancelled') and due_date is not null
        ) as next_due_date
    from filtered_milestones
    group by project_id, project_name
    order by overdue_milestones desc, upcoming_milestones desc, project_name asc
"#;

const CLIENT_CHANGE_REQUEST_STATUS_SQL: &str = r#"
    select cr.status, count(*) as total_requests
    from change_requests cr
    where cr.user_id = $1
      and cr.client_id = $2
      and ($3::uuid is null or cr.project_id = $3)
      and ($4::varchar is null or cr.status = $4)
      and ($5::date is null or cr.created_at::date >= $5)
      and ($6::date is null or cr.created_at::date <= $6)
    group by cr.status
    order by total_requests desc, cr.status asc
"#;

const CLIENT_APPROVED_CHANGE_VALUE_SQL: &str = r#"
    select currency_type, total_approved_changes, total_approved_value
    from vw_client_dash_approved_change_value
    where user_id = $1 and client_id = $2
    order by currency_type asc
"#;

const CLIENT_PROJECT_PORTFOLIO_SQL: &str = r#"
    with next_milestone as (
        select
            m.project_id,
            m.name as next_milestone_name,
            m.due_date::date as next_milestone_due_date,
            row_number() over (
                partition by m.project_id
                order by m.due_date asc nulls last, m.created_at asc
            ) as rn
        from milestones m
        where m.status not in ('approved', 'released', 'cancelled')
    ),
    milestone_health as (
        select
            p.id as project_id,
            count(m.id) as total_milestones,
            sum(case when m.status in ('approved', 'released') then 1 else 0 end) as completed_milestones,
            sum(
                case when m.status not in ('approved', 'released', 'cancelled')
                     and m.due_date is not null and m.due_date < now()
                     then 1 else 0 end
            ) as overdue_milestones,
            sum(
                case when m.status not in ('approved', 'released', 'cancelled')
                     and m.due_date is not null
                     and m.due_date >= now()
                     and m.due_date < now() + interval '7 days'
                     then 1 else 0 end
            ) as upcoming_milestones
        from projects p
        left join milestones m on m.project_id = p.id
        group by p.id
    )
    select
        p.id as project_id,
        p.name as project_name,
        p.status as project_status,
        p.billable,
        pb.currency_type,
        pb.billing_type,
        pb.hourly_rate,
        pb.fixed_price,
        pb.retainer_amount,
        pb.payment_day,
        coalesce(mh.total_milestones, 0) as total_milestones,
        coalesce(mh.completed_milestones, 0) as completed_milestones,
        coalesce(mh.overdue_milestones, 0) as overdue_milestones,
        coalesce(mh.upcoming_milestones, 0) as upcoming_milestones,
        nm.next_milestone_name,
        nm.next_milestone_due_date
    from projects p
    left join project_billing pb on pb.project_id = p.id and pb.user_id = p.user_id
    left join milestone_health mh on mh.project_id = p.id
    left join next_milestone nm on nm.project_id = p.id and nm.rn = 1
    where p.user_id = $1
      and p.client_id = $2
      and ($3::varchar is null or p.status = $3)
      and ($4::boolean is null or p.billable = $4)
      and ($5::varchar is null or pb.billing_type = $5)
      and ($6::varchar is null or pb.currency_type = $6)
      and ($7::boolean is not true or coalesce(mh.overdue_milestones, 0) > 0)
      and ($8::boolean is not true or coalesce(mh.upcoming_milestones, 0) > 0)
    order by overdue_milestones desc, upcoming_milestones desc, project_name asc
"#;

pub struct DashboardService;

impl DashboardService {
    pub async fn get_overview(db: &impl ConnectionTrait, user_id: Uuid) -> Result<DashboardDto, AppError> {
        let monthly_revenue = Self::query_monthly_revenue(db, user_id, &MonthlyRevenueFiltersDto::default()).await?;
        let accounts_receivable =
            Self::query_accounts_receivable(db, user_id, &AccountsReceivableFiltersDto::default()).await?;
        let time_by_activity = Self::query_time_by_activity(db, user_id, &TimeByActivityFiltersDto::default()).await?;
        let billable_ratio_daily = Self::query_billable_ratio(db, user_id, &BillableRatioFiltersDto::default()).await?;
        let scope_creep_value =
            Self::query_simple::<ScopeCreepValueRow>(db, SCOPE_CREEP_VALUE_SQL, vec![Value::from(user_id)]).await?;
        let task_bottlenecks = Self::query_task_bottlenecks(db, user_id, &TaskBottlenecksFiltersDto::default()).await?;

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
            Self::query_client_project_status(db, user_id, client_id, &ClientProjectStatusFiltersDto::default())
                .await?;
        let milestone_health =
            Self::query_client_milestone_health(db, user_id, client_id, &ClientMilestoneHealthFiltersDto::default())
                .await?;
        let change_request_status = Self::query_client_change_request_status(
            db,
            user_id,
            client_id,
            &ClientChangeRequestStatusFiltersDto::default(),
        )
        .await?;
        let approved_change_value = Self::query_simple::<ClientApprovedChangeValueRow>(
            db,
            CLIENT_APPROVED_CHANGE_VALUE_SQL,
            vec![Value::from(user_id), Value::from(client_id)],
        )
        .await?;
        let project_portfolio =
            Self::query_client_project_portfolio(db, user_id, client_id, &ClientProjectPortfolioFiltersDto::default())
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

    pub async fn query_monthly_revenue(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filters: &MonthlyRevenueFiltersDto,
    ) -> Result<Vec<MonthlyRevenueRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(filters.date_from),
            Value::from(filters.date_to),
            Value::from(filters.currency_type.clone()),
            Value::from(filters.client_id),
            Value::from(filters.project_id),
        ];
        Self::query_simple::<MonthlyRevenueRow>(db, MONTHLY_REVENUE_SQL, values).await
    }

    pub async fn query_accounts_receivable(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filters: &AccountsReceivableFiltersDto,
    ) -> Result<Vec<AccountsReceivableRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(filters.client_id),
            Value::from(filters.currency_type.clone()),
            Value::from(filters.due_date_from),
            Value::from(filters.due_date_to),
            Value::from(filters.overdue_only),
            Value::from(filters.min_outstanding_balance),
        ];
        Self::query_simple::<AccountsReceivableRow>(db, ACCOUNTS_RECEIVABLE_SQL, values).await
    }

    pub async fn query_time_by_activity(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filters: &TimeByActivityFiltersDto,
    ) -> Result<Vec<TimeByActivityRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(filters.date_from),
            Value::from(filters.date_to),
            Value::from(filters.activity_id),
            Value::from(filters.billable),
            Value::from(filters.client_id),
            Value::from(filters.project_id),
        ];
        Self::query_simple::<TimeByActivityRow>(db, TIME_BY_ACTIVITY_SQL, values).await
    }

    pub async fn query_billable_ratio(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filters: &BillableRatioFiltersDto,
    ) -> Result<Vec<BillableRatioDailyRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(filters.date_from),
            Value::from(filters.date_to),
            Value::from(filters.client_id),
            Value::from(filters.project_id),
            Value::from(filters.activity_id),
        ];
        Self::query_simple::<BillableRatioDailyRow>(db, BILLABLE_RATIO_DAILY_SQL, values).await
    }

    pub async fn query_task_bottlenecks(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filters: &TaskBottlenecksFiltersDto,
    ) -> Result<Vec<TaskBottleneckRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(filters.client_id),
            Value::from(filters.project_id),
            Value::from(filters.status.clone()),
            Value::from(filters.priority.map(i32::from)),
            Value::from(filters.overdue_only),
        ];
        Self::query_simple::<TaskBottleneckRow>(db, TASK_BOTTLENECKS_SQL, values).await
    }

    pub async fn query_client_project_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        filters: &ClientProjectStatusFiltersDto,
    ) -> Result<Vec<ClientProjectStatusRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(client_id),
            Value::from(filters.project_status.clone()),
            Value::from(filters.billable),
            Value::from(filters.category_id),
        ];
        Self::query_simple::<ClientProjectStatusRow>(db, CLIENT_PROJECT_STATUS_SQL, values).await
    }

    pub async fn query_client_milestone_health(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        filters: &ClientMilestoneHealthFiltersDto,
    ) -> Result<Vec<ClientMilestoneHealthRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(client_id),
            Value::from(filters.project_id),
            Value::from(filters.milestone_status.clone()),
            Value::from(filters.due_date_from),
            Value::from(filters.due_date_to),
            Value::from(filters.overdue_only),
            Value::from(filters.upcoming_only),
        ];
        Self::query_simple::<ClientMilestoneHealthRow>(db, CLIENT_MILESTONE_HEALTH_SQL, values).await
    }

    pub async fn query_client_change_request_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        filters: &ClientChangeRequestStatusFiltersDto,
    ) -> Result<Vec<ClientChangeRequestStatusRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(client_id),
            Value::from(filters.project_id),
            Value::from(filters.status.clone()),
            Value::from(filters.date_from),
            Value::from(filters.date_to),
        ];
        Self::query_simple::<ClientChangeRequestStatusRow>(db, CLIENT_CHANGE_REQUEST_STATUS_SQL, values).await
    }

    pub async fn query_client_project_portfolio(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        filters: &ClientProjectPortfolioFiltersDto,
    ) -> Result<Vec<ClientProjectPortfolioRow>, AppError> {
        let values = vec![
            Value::from(user_id),
            Value::from(client_id),
            Value::from(filters.project_status.clone()),
            Value::from(filters.billable),
            Value::from(filters.billing_type.clone()),
            Value::from(filters.currency_type.clone()),
            Value::from(filters.overdue_milestones_only),
            Value::from(filters.upcoming_milestones_only),
        ];
        Self::query_simple::<ClientProjectPortfolioRow>(db, CLIENT_PROJECT_PORTFOLIO_SQL, values).await
    }

    async fn query_simple<T>(db: &impl ConnectionTrait, sql: &str, values: Vec<Value>) -> Result<Vec<T>, AppError>
    where
        T: FromQueryResult,
    {
        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, values);
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
