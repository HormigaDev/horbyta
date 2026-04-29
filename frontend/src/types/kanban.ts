import type { TaskStatus, TaskSummaryDto } from './task';

export interface KanbanColumnDefinition {
  id: TaskStatus;
  title: string;
}

export type KanbanTask = TaskSummaryDto & {
  has_time_entries?: boolean;
  time_entry_count?: number;
};

export interface TaskMovePayload {
  taskId: string;
  newColumnId: TaskStatus;
}
