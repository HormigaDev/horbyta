export interface MonthlyRevenueFiltersDto {
  date_from: string | null;
  date_to: string | null;
  currency_type: string | null;
  client_id: string | null;
  project_id: string | null;
}

export interface AccountsReceivableFiltersDto {
  client_id: string | null;
  currency_type: string | null;
  due_date_from: string | null;
  due_date_to: string | null;
  overdue_only: boolean | null;
  min_outstanding_balance: string | null;
}

export interface TimeByActivityFiltersDto {
  date_from: string | null;
  date_to: string | null;
  activity_id: string | null;
  billable: boolean | null;
  client_id: string | null;
  project_id: string | null;
}

export interface BillableRatioFiltersDto {
  date_from: string | null;
  date_to: string | null;
  client_id: string | null;
  project_id: string | null;
  activity_id: string | null;
}

export interface TaskBottlenecksFiltersDto {
  client_id: string | null;
  project_id: string | null;
  status: string | null;
  priority: number | null;
  overdue_only: boolean | null;
}

export interface ClientProjectStatusFiltersDto {
  project_status: string | null;
  billable: boolean | null;
  category_id: string | null;
}

export interface ClientMilestoneHealthFiltersDto {
  project_id: string | null;
  milestone_status: string | null;
  due_date_from: string | null;
  due_date_to: string | null;
  overdue_only: boolean | null;
  upcoming_only: boolean | null;
}

export interface ClientChangeRequestStatusFiltersDto {
  project_id: string | null;
  status: string | null;
  date_from: string | null;
  date_to: string | null;
}

export interface ClientProjectPortfolioFiltersDto {
  project_status: string | null;
  billable: boolean | null;
  billing_type: string | null;
  currency_type: string | null;
  overdue_milestones_only: boolean | null;
  upcoming_milestones_only: boolean | null;
}

export type DashboardChartKey =
  | 'monthlyRevenue'
  | 'accountsReceivable'
  | 'timeByActivity'
  | 'billableRatio'
  | 'taskBottlenecks';

export type ClientDashboardChartKey =
  | 'clientProjectStatus'
  | 'clientMilestoneHealth'
  | 'clientChangeRequestStatus'
  | 'clientProjectPortfolio';
