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
import type { MarkdownPendingFile } from 'src/types/markdown';

const buildTaskBody = (
  data: CreateTaskDto | UpdateTaskDto,
  files: readonly MarkdownPendingFile[],
): string | FormData => {
  if (files.length === 0) {
    return JSON.stringify(data);
  }

  const formData = new FormData();
  formData.append('payload', JSON.stringify(data));

  files.forEach((file) => {
    formData.append(`markdown_file:${file.id}`, file.file);
  });

  return formData;
};

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

  create: (data: CreateTaskDto, files: readonly MarkdownPendingFile[] = []): Promise<TaskDetailDto> =>
    apiFetch('/tasks', {
      method: 'POST',
      body: buildTaskBody(data, files),
    }),

  getById: (id: string): Promise<TaskDetailDto> =>
    apiFetch(`/tasks/${id}`, {
      method: 'GET',
    }),

  update: (
    id: string,
    data: UpdateTaskDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<TaskDetailDto> =>
    apiFetch(`/tasks/${id}`, {
      method: 'PUT',
      body: buildTaskBody(data, files),
    }),

  changeStatus: (id: string, status: TaskStatus): Promise<void> =>
    apiFetch(`/tasks/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    }),

  updateTaskStatus: (taskId: string, statusId: TaskStatus): Promise<void> =>
    apiFetch(`/tasks/${taskId}`, {
      method: 'PATCH',
      body: JSON.stringify({ status: statusId }),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/tasks/${id}`, {
      method: 'DELETE',
    }),
};
