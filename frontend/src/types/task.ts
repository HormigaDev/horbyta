export type TaskStatus = 'todo' | 'in_progress' | 'review' | 'done';

export interface TaskSummary {
  id: string;
  project_id: string | null;
  milestone_id: string | null;
  title: string;
  status: TaskStatus;
  priority: number;
  complexity: number;
  due_date: string | null;
  project_name: string | null;
  milestone_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface TaskDetail {
  id: string;
  user_id: string;
  project_id: string | null;
  parent_id: string | null;
  milestone_id: string | null;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: number;
  complexity: number;
  block_parent: boolean;
  due_date: string | null;
  project_name: string | null;
  milestone_name: string | null;
  created_at: string;
  updated_at: string;
  tag_ids: string[];
  tags: LookupOption[];
}

export interface TaskDto {
  project_id: string | null;
  parent_id: string | null;
  milestone_id: string | null;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: number;
  complexity: number;
  block_parent: boolean;
  due_date: string | null;
  tag_ids: string[];
}

export interface LookupOption {
  id: string;
  name: string;
}

export interface MilestoneLookup {
  id: string;
  project_id: string;
  name: string;
  due_date: string | null;
}

export interface TasksViewResponse {
  items: TaskSummary[];
  total: number;
  projects: LookupOption[];
  milestones: MilestoneLookup[];
  tags: LookupOption[];
}
