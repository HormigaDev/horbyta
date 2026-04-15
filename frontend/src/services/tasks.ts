import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  CreateTaskDto,
  TaskDetailDto,
  TaskStatus,
  TaskSummaryDto,
  TasksViewResponse,
  UpdateTaskDto,
} from 'src/types/task';

export const TasksService = {
  view: (): Promise<TasksViewResponse> =>
    apiFetch('/tasks/view', {
      method: 'GET',
    }),

  search: (payload: QueryPayload): Promise<PaginatedResponse<TaskSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/tasks/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CreateTaskDto): Promise<TaskDetailDto> =>
    apiFetch('/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<TaskDetailDto> =>
    apiFetch(`/tasks/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateTaskDto): Promise<TaskDetailDto> =>
    apiFetch(`/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  changeStatus: (id: string, status: TaskStatus): Promise<void> =>
    apiFetch(`/tasks/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/tasks/${id}`, {
      method: 'DELETE',
    }),
};
