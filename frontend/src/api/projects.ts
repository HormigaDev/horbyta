import type { QueryPayload, PaginatedResponse } from '../types/query';
import type {
  Project,
  ProjectSummary,
  CreateProjectDto,
  UpdateProjectDto,
  UpdateProjectCategoryDto,
} from '../types/project';
import { apiFetch } from '../boot/fetch';

export const projectsApi = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ProjectSummary>> =>
    apiFetch('/projects/search', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  create: (data: CreateProjectDto): Promise<Project> =>
    apiFetch('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<Project> =>
    apiFetch(`/projects/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateProjectDto): Promise<Project> =>
    apiFetch(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  updateCategory: (id: string, data: UpdateProjectCategoryDto): Promise<Project> =>
    apiFetch(`/projects/${id}/category`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/projects/${id}`, {
      method: 'DELETE',
    }),
};
