import type { TaskStatus, TaskSummaryDto } from './task';

/** Cada columna es un estado del usuario (id del estado). */
export interface KanbanColumnDefinition {
  id: string;
  title: string;
  color: string;
  category: TaskStatus;
}

export type KanbanTask = TaskSummaryDto & {
  has_time_entries?: boolean;
  time_entry_count?: number;
};

export interface TaskMovePayload {
  taskId: string;
  newColumnId: string;
}
