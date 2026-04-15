export type ProjectStatus = 'open' | 'closed' | 'cancelled';
export type ProjectMilestoneStatus = 'open' | 'review' | 'approved' | 'released' | 'cancelled';

export interface ProjectSummaryDto {
  id: string;
  record_number: bigint;
  name: string;
  status: ProjectStatus;
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
  category_id: string | null;
  client_id: string | null;
  category_name: string | null;
  client_name: string | null;
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
  milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectDto {
  name: string;
  description?: string | null;
  status: ProjectStatus;
  category_id?: string | null;
  client_id?: string | null;
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
  milestones: ProjectDetailMilestoneDto[];
  created_at: string;
  updated_at: string;
}
