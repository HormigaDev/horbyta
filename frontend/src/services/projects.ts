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
import type { MarkdownPendingFile } from 'src/types/markdown';

const buildProjectBody = (
  data: CreateProjectDto | UpdateProjectDto,
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

export const ProjectsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ProjectSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/projects/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (
    data: CreateProjectDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<ProjectMutationResultDto> =>
    apiFetch('/projects', {
      method: 'POST',
      body: buildProjectBody(data, files),
    }),

  getById: (id: string): Promise<ProjectDetailDto> =>
    apiFetch(`/projects/${id}`, {
      method: 'GET',
    }),

  update: (
    id: string,
    data: UpdateProjectDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<ProjectMutationResultDto> =>
    apiFetch(`/projects/${id}`, {
      method: 'PUT',
      body: buildProjectBody(data, files),
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
