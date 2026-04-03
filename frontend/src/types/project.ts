export interface ProjectSummary {
  id: string;
  name: string;
  category_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface Project {
  id: string;
  user_id: string;
  name: string;
  description: string | null;
  category_id: string | null;
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
  category_id?: string | null;
  milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectDto {
  name: string;
  description?: string | null;
  category_id?: string | null;
  new_milestones?: CreateMilestoneSubDto[] | null;
}

export interface UpdateProjectCategoryDto {
  category_id: string | null;
}
