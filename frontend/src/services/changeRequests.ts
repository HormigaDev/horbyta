import { apiFetch } from 'src/boot/fetch';
import { buildMultipartPayload } from 'src/services/attachments';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { MarkdownPendingFile } from 'src/types/markdown';
import type {
  AssessChangeRequestDto,
  AssignChangeRequestTaskDto,
  ChangeRequestDetailDto,
  ChangeRequestSummaryDto,
  CreateChangeRequestDto,
  UpdateChangeRequestDto,
} from 'src/types/changeRequest';

export const ChangeRequestsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ChangeRequestSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/change-requests/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (
    data: CreateChangeRequestDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<ChangeRequestDetailDto> =>
    apiFetch('/change-requests', {
      method: 'POST',
      body: buildMultipartPayload(data, files),
    }),

  getById: (id: string): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}`, {
      method: 'GET',
    }),

  update: (
    id: string,
    data: UpdateChangeRequestDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}`, {
      method: 'PUT',
      body: buildMultipartPayload(data, files),
    }),

  assignTask: (id: string, data: AssignChangeRequestTaskDto): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}/task`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  submit: (id: string): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}/submit`, {
      method: 'PATCH',
    }),

  assess: (id: string, data: AssessChangeRequestDto): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}/assess`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  markImplemented: (id: string): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}/implemented`, {
      method: 'PATCH',
    }),

  cancel: (id: string): Promise<ChangeRequestDetailDto> =>
    apiFetch(`/change-requests/${id}/cancel`, {
      method: 'PATCH',
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/change-requests/${id}`, {
      method: 'DELETE',
    }),
};
