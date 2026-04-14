import { apiFetchClient } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { Milestone } from 'src/types/milestone';

export const ClientMilestonesService = {
  searchByProject: (
    projectId: string,
    payload: QueryPayload,
  ): Promise<PaginatedResponse<Milestone>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetchClient(
      `/milestones/client/search-by-project/${projectId}?${params.toString()}`,
      {
        method: 'GET',
      },
    );
  },
};
