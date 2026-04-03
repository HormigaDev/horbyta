import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { Milestone, MilestoneDto } from 'src/types/milestone';

export const MilestonesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<Milestone>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/milestones/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: MilestoneDto): Promise<Milestone> =>
    apiFetch('/milestones', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: MilestoneDto): Promise<Milestone> =>
    apiFetch(`/milestones/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/milestones/${id}`, {
      method: 'DELETE',
    }),
};
