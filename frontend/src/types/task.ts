import type { TagSummaryDto } from './tag';

export type TaskStatus = 'todo' | 'in_progress' | 'review' | 'done';

export interface TaskSummaryDto {
  id: string;
  task_number: bigint;
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

export interface TaskDetailDto {
  id: string;
  task_number: bigint;
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
  tags: TagSummaryDto[];
}

export interface CreateTaskDto {
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

export interface UpdateTaskDto {
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

export interface LookupOptionDto {
  id: string;
  name: string;
}

export interface MilestoneLookupDto {
  id: string;
  project_id: string;
  name: string;
  due_date: string | null;
}

export interface TasksViewResponse {
  items: TaskSummaryDto[];
  total: number;
  projects: LookupOptionDto[];
  milestones: MilestoneLookupDto[];
  tags: TagSummaryDto[];
}
