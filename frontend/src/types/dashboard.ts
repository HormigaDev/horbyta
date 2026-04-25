import type { ChangeRequestStatus } from './changeRequest';
import type { ProjectBillingType, ProjectStatus } from './project';

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

export interface ClientDashboardKpisDto {
  active_projects_total: number;
  overdue_milestones_total: number;
  open_change_requests_total: number;
  approved_change_value: CurrencyAmountDto[];
}

export interface ClientProjectStatusRow {
  status: ProjectStatus;
  total_projects: number;
}

export interface ClientMilestoneHealthRow {
  project_id: string;
  project_name: string;
  total_milestones: number;
  completed_milestones: number;
  overdue_milestones: number;
  upcoming_milestones: number;
  next_due_date: string | null;
}

export interface ClientChangeRequestStatusRow {
  status: ChangeRequestStatus;
  total_requests: number;
}

export interface ClientApprovedChangeValueRow {
  currency_type: string;
  total_approved_changes: number;
  total_approved_value: string;
}

export interface ClientProjectPortfolioRow {
  project_id: string;
  project_name: string;
  project_status: ProjectStatus;
  billable: boolean;
  currency_type: string | null;
  billing_type: ProjectBillingType | null;
  hourly_rate: string | null;
  fixed_price: string | null;
  retainer_amount: string | null;
  payment_day: number | null;
  total_milestones: number;
  completed_milestones: number;
  overdue_milestones: number;
  upcoming_milestones: number;
  next_milestone_name: string | null;
  next_milestone_due_date: string | null;
}

export interface ClientDashboardDto {
  kpis: ClientDashboardKpisDto;
  project_status: ClientProjectStatusRow[];
  milestone_health: ClientMilestoneHealthRow[];
  change_request_status: ClientChangeRequestStatusRow[];
  approved_change_value: ClientApprovedChangeValueRow[];
  project_portfolio: ClientProjectPortfolioRow[];
}
