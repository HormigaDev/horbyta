import { apiFetch } from 'src/boot/fetch';
import type { TaskStatus, TaskStatusItem } from 'src/types/task';

export interface SaveTaskStatusDto {
  name: string;
  category: TaskStatus;
  color: string;
}

export const TaskStatusesService = {
  list: (): Promise<TaskStatusItem[]> => apiFetch('/task-statuses'),
  create: (data: SaveTaskStatusDto): Promise<TaskStatusItem> =>
    apiFetch('/task-statuses', { method: 'POST', body: JSON.stringify(data) }),
  update: (id: string, data: SaveTaskStatusDto): Promise<TaskStatusItem> =>
    apiFetch(`/task-statuses/${id}`, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (id: string): Promise<void> => apiFetch(`/task-statuses/${id}`, { method: 'DELETE' }),
  reorder: (ids: string[]): Promise<TaskStatusItem[]> =>
    apiFetch('/task-statuses/order', { method: 'PUT', body: JSON.stringify({ ids }) }),
};
