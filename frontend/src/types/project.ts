export type ProjectStatus = 'open' | 'closed' | 'cancelled';
export type ProjectMilestoneStatus = 'open' | 'review' | 'approved' | 'released' | 'cancelled';
export type ProjectBillingType = 'fixed' | 'hourly' | 'retainer';

export interface ProjectBillingDto {
  id: string;
  user_id: string;
  project_id: string;
  currency_type: string;
  billing_type: ProjectBillingType;
  hourly_rate: string | null;
  fixed_price: string | null;
  retainer_amount: string | null;
  payment_day: number | null;
  created_at: string;
  updated_at: string;
}

export interface UpsertProjectBillingDto {
  currency_type: string;
  billing_type: ProjectBillingType;
  hourly_rate?: string | null;
  fixed_price?: string | null;
  retainer_amount?: string | null;
  payment_day?: number | null;
}

export interface ProjectSummaryDto {
  id: string;
  record_number: bigint;
  name: string;
  status: ProjectStatus;
  billable: boolean;
  category_id?: string | null;
  category_name: string | null;
  client_id?: string | null;
  client_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectDetailMilestoneDto {
  id: string;
  name: string;
  description: string | null;
  status: ProjectMilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectDetailDto {
  id: string;
  record_number: bigint;
  name: string;
  description: string | null;
  status: ProjectStatus;
  billable: boolean;
  category_id: string | null;
  client_id: string | null;
  category_name: string | null;
  client_name: string | null;
  project_billing: ProjectBillingDto | null;
  milestones: ProjectDetailMilestoneDto[];
  created_at: string;
  updated_at: string;
}

export interface CreateMilestoneSubDto {
  name: string;
  due_date?: string | null;
}

export interface CreateProjectDto {
  name: string;
  description?: string | null;
  status?: ProjectStatus | null;
  category_id?: string | null;
  client_id?: string | null;
  billable: boolean;
  project_billing?: UpsertProjectBillingDto | null;
  milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectDto {
  name: string;
  description?: string | null;
  status: ProjectStatus;
  category_id?: string | null;
  client_id?: string | null;
  billable: boolean;
  project_billing?: UpsertProjectBillingDto | null;
  new_milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectCategoryDto {
  category_id: string | null;
}

export interface ProjectMutationResultDto {
  id: string;
  record_number: bigint;
  user_id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  billable: boolean;
  category_id: string | null;
  client_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectFormStateDto {
  id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  billable: boolean;
  category_id: string | null;
  client_id: string | null;
}

export interface ClientProjectSummaryDto {
  id: string;
  record_number: bigint;
  name: string;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}

export interface ClientProjectDetailDto {
  id: string;
  record_number: bigint;
  name: string;
  description: string | null;
  status: ProjectStatus;
  billable: boolean;
  project_billing: ProjectBillingDto | null;
  milestones: ProjectDetailMilestoneDto[];
  created_at: string;
  updated_at: string;
}
