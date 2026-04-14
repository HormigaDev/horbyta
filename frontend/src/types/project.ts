export type ProjectStatus = 'open' | 'closed' | 'cancelled';

export interface ProjectSummary {
  id: string;
  name: string;
  status: ProjectStatus;
  category_name: string | null;
  client_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface Project {
  id: string;
  user_id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  category_id: string | null;
  client_id: string | null;
  category_name?: string | null;
  client_name?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectDetailDto {
  id: string;
  user_id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  category_name: string | null;
  client_name: string | null;
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
  status?: ProjectStatus;
  category_id?: string | null;
  client_id?: string | null;
  new_milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectCategoryDto {
  category_id: string | null;
}

export interface ClientProjectSummary {
  id: string;
  name: string;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}

export interface ClientProjectDetail {
  id: string;
  name: string;
  description: string | null;
  status: ProjectStatus;
  created_at: string;
  updated_at: string;
}
