import { apiFetchClient } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { ClientProjectDetailDto, ClientProjectSummaryDto } from 'src/types/project';

export const ClientProjectsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ClientProjectSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetchClient(`/projects/client-search?${params.toString()}`, {
      method: 'GET',
    });
  },

  getById: (id: string): Promise<ClientProjectDetailDto> =>
    apiFetchClient(`/projects/${id}/client`, {
      method: 'GET',
    }),
};
