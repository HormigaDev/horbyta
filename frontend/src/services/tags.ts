import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { Tag, TagDto } from 'src/types/tag';

export const TagsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<Tag>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/tags/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: TagDto): Promise<Tag> =>
    apiFetch('/tags', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<Tag> =>
    apiFetch(`/tags/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: TagDto): Promise<Tag> =>
    apiFetch(`/tags/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/tags/${id}`, {
      method: 'DELETE',
    }),
};
