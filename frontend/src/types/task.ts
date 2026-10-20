import type { TagSummaryDto } from './tag';

/** Categoría del estado: el significado que usa el sistema (alcance, tablero, panel). */
export type TaskStatus = 'todo' | 'in_progress' | 'review' | 'done' | 'cancelled';

/** Estado editable del usuario. `key` marca los estados por defecto. */
export interface TaskStatusItem {
  id: string;
  key: TaskStatus | null;
  name: string;
  category: TaskStatus;
  color: string;
  position: number;
  task_count: number;
}

/** Datos del estado concreto que acompañan a cada tarea. */
export interface TaskStatusFields {
  status: TaskStatus;
  status_id: string;
  status_name: string | null;
  status_color: string | null;
  status_key: TaskStatus | null;
}

export interface TaskSummaryDto extends TaskStatusFields {
  id: string;
  record_number: bigint;
  project_id: string | null;
  milestone_id: string | null;
  title: string;
  priority: number;
  complexity: number;
  due_date: string | null;
  project_name: string | null;
  milestone_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface TaskDetailDto extends TaskStatusFields {
  id: string;
  record_number: bigint;
  user_id: string;
  project_id: string | null;
  parent_id: string | null;
  milestone_id: string | null;
  title: string;
  description: string | null;
  priority: number;
  complexity: number;
  block_parent: boolean;
  due_date: string | null;
  project_name: string | null;
  milestone_name: string | null;
  parent_title: string | null;
  parent_record_number: bigint | null;
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
  /** Estado elegido; si falta, la base usa el primero de la categoría `status`. */
  status_id?: string | null;
  status?: TaskStatus;
  priority: number;
  complexity: number;
  block_parent: boolean;
  due_date: string | null;
  tag_ids: string[];
  /** Origen en el alcance: null = incluida en el acuerdo; id = extra de esa solicitud. */
  scope_origin?: { change_request_id: string | null };
}

export interface UpdateTaskDto {
  project_id: string | null;
  parent_id: string | null;
  milestone_id: string | null;
  title: string;
  description: string | null;
  /** Estado elegido; si falta, la base usa el primero de la categoría `status`. */
  status_id?: string | null;
  status?: TaskStatus;
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
