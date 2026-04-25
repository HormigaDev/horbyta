export interface CurrencyAmountDto {
  currency_type: string;
  amount: string;
}

export interface DashboardKpisDto {
  scope_creep_recovered: CurrencyAmountDto[];
  accounts_receivable_total: CurrencyAmountDto[];
  overdue_tasks_total: number;
}

export interface MonthlyRevenueRow {
  currency_type: string;
  month_date: string;
  total_revenue: string;
}

export interface AccountsReceivableRow {
  client_name: string;
  currency_type: string;
  outstanding_balance: string;
  oldest_due_date: string | null;
}

export interface TimeByActivityRow {
  activity_name: string;
  billable: boolean;
  week_date: string;
  total_hours: string;
}

export interface BillableRatioDailyRow {
  work_date: string;
  billable_hours: string;
  no_billable_hours: string;
  total_hours: string;
}

export interface ScopeCreepValueRow {
  currency_type: string;
  total_approved_changes: number;
  total_additional_revenue: string;
  total_days_extended: number;
}

export interface TaskBottleneckRow {
  project_id: string;
  project_name: string;
  total_tasks: number;
  tasks_todo: number;
  tasks_in_progress: number;
  tasks_in_review: number;
  overdue_tasks: number;
}

export interface DashboardDto {
  kpis: DashboardKpisDto;
  monthly_revenue: MonthlyRevenueRow[];
  accounts_receivable: AccountsReceivableRow[];
  time_by_activity: TimeByActivityRow[];
  billable_ratio_daily: BillableRatioDailyRow[];
  scope_creep_value: ScopeCreepValueRow[];
  task_bottlenecks: TaskBottleneckRow[];
}
