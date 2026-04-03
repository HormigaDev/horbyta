export interface Milestone {
  id: string;
  project_id: string;
  name: string;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface MilestoneDto {
  project_id: string;
  name: string;
  due_date: string | null;
}
