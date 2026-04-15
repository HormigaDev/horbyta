import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { CreateTagDto, TagDetailDto, TagSummaryDto, UpdateTagDto } from 'src/types/tag';

export const TagsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<TagSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/tags/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CreateTagDto): Promise<TagDetailDto> =>
    apiFetch('/tags', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<TagDetailDto> =>
    apiFetch(`/tags/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateTagDto): Promise<TagDetailDto> =>
    apiFetch(`/tags/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/tags/${id}`, {
      method: 'DELETE',
    }),
};
