import { apiFetchClient } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { ClientMilestone } from 'src/types/milestone';

export const ClientMilestonesService = {
  searchByProject: (
    projectId: string,
    payload: QueryPayload,
  ): Promise<PaginatedResponse<ClientMilestone>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetchClient(
      `/milestones/client/search-by-project/${projectId}?${params.toString()}`,
      {
        method: 'GET',
      },
    );
  },

  getByIdFromProject: async (projectId: string, id: string): Promise<ClientMilestone> => {
    const payload: QueryPayload = {
      page: 1,
      limit: 1,
      order_by: 'created_at',
      order_asc: false,
      filters: [{ column: 'id', operator: '=', value: id }],
    };

    const response = await ClientMilestonesService.searchByProject(projectId, payload);
    const milestone = response.items[0];

    if (!milestone) {
      throw new Error('Milestone no encontrado');
    }

    return milestone;
  },
};
