import type { QueryPayload, PaginatedResponse } from '../types/query';
import type {
  ProjectMutationResultDto,
  ProjectSummaryDto,
  CreateProjectDto,
  UpdateProjectDto,
  UpdateProjectCategoryDto,
  ProjectDetailDto,
} from '../types/project';
import { apiFetch } from '../boot/fetch';

const buildProjectCreateBody = (
  data: CreateProjectDto,
  files: readonly File[],
): string | FormData => {
  if (files.length === 0) {
    return JSON.stringify(data);
  }

  const formData = new FormData();
  formData.append('payload', JSON.stringify(data));

  files.forEach((file) => {
    formData.append('files', file);
  });

  return formData;
};

export const ProjectsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ProjectSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/projects/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (
    data: CreateProjectDto,
    files: readonly File[] = [],
  ): Promise<ProjectMutationResultDto> =>
    apiFetch('/projects', {
      method: 'POST',
      body: buildProjectCreateBody(data, files),
    }),

  getById: (id: string): Promise<ProjectDetailDto> =>
    apiFetch(`/projects/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateProjectDto): Promise<ProjectMutationResultDto> =>
    apiFetch(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  updateCategory: (id: string, data: UpdateProjectCategoryDto): Promise<ProjectMutationResultDto> =>
    apiFetch(`/projects/${id}/category`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/projects/${id}`, {
      method: 'DELETE',
    }),
};
