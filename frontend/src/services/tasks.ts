import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { TaskDetail, TaskDto, TaskSummary, TasksViewResponse } from 'src/types/task';

export const TasksService = {
  view: (): Promise<TasksViewResponse> =>
    apiFetch('/tasks/view', {
      method: 'GET',
    }),

  search: (payload: QueryPayload): Promise<PaginatedResponse<TaskSummary>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/tasks/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: TaskDto): Promise<TaskDetail> =>
    apiFetch('/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<TaskDetail> =>
    apiFetch(`/tasks/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: TaskDto): Promise<TaskDetail> =>
    apiFetch(`/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/tasks/${id}`, {
      method: 'DELETE',
    }),
};
