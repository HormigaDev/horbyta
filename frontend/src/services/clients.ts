import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  Client,
  ClientSummary,
  CreateClientDto,
  UpdateClientDto,
  UpdateClientStatusDto,
} from 'src/types/clients';

export const ClientsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<ClientSummary>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/clients/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CreateClientDto): Promise<Client> =>
    apiFetch('/clients', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<Client> => apiFetch(`/clients/${id}`, { method: 'GET' }),

  update: (id: string, data: UpdateClientDto): Promise<Client> =>
    apiFetch(`/clients/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  updateStatus: (id: string, data: UpdateClientStatusDto): Promise<Client> =>
    apiFetch(`/clients/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<void> => apiFetch(`/clients/${id}`, { method: 'DELETE' }),
};
