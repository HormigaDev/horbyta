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

export interface CreateProjectDto {
  name: string;
  description?: string | null;
  category_id?: string | null;
}

export type UpdateProjectDto = CreateProjectDto;

export interface UpdateProjectCategoryDto {
  category_id: string | null;
}
