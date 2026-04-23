import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  ActivityDetailDto,
  ActivitySummaryDto,
  CreateActivityDto,
  UpdateActivityDto,
} from 'src/types/activity';

export const ActivitiesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ActivitySummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/activities/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CreateActivityDto): Promise<ActivityDetailDto> =>
    apiFetch('/activities', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<ActivityDetailDto> =>
    apiFetch(`/activities/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateActivityDto): Promise<ActivityDetailDto> =>
    apiFetch(`/activities/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: async (id: string): Promise<void> => {
    await apiFetch(`/activities/${id}`, {
      method: 'DELETE',
    });
  },
};
