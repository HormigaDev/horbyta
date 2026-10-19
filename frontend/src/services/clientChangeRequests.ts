import { apiFetchClient } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  ClientApproveChangeRequestDto,
  ClientCancelChangeRequestDto,
  ClientChangeRequestDetailDto,
  ClientChangeRequestSummaryDto,
  ClientCreateChangeRequestDto,
  ClientUpdateChangeRequestDto,
} from 'src/types/changeRequest';

export const ClientChangeRequestsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ClientChangeRequestSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetchClient(`/change-requests/client/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: ClientCreateChangeRequestDto): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient('/change-requests/client', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient(`/change-requests/client/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: ClientUpdateChangeRequestDto): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient(`/change-requests/client/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  cancel: (id: string, data: ClientCancelChangeRequestDto): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient(`/change-requests/client/${id}/cancel`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  approve: (
    id: string,
    data: ClientApproveChangeRequestDto,
  ): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient(`/change-requests/client/${id}/approve`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  decline: (
    id: string,
    decline_reason: string,
    version: number,
    decline_comment: string | null = null,
  ): Promise<ClientChangeRequestDetailDto> =>
    apiFetchClient(`/change-requests/client/${id}/decline`, {
      method: 'PATCH',
      body: JSON.stringify({ decline_reason, version, decline_comment }),
    }),
};
