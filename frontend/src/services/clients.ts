import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  ClientAuthStatusDto,
  Client,
  ClientMagicLinkDto,
  ClientSummary,
  CreateClientDto,
  SaveClientCredentialsDto,
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

  getAuthStatus: (id: string): Promise<ClientAuthStatusDto> =>
    apiFetch(`/clients/${id}/auth`, {
      method: 'GET',
    }),

  activateCredentials: (id: string, data: SaveClientCredentialsDto): Promise<ClientAuthStatusDto> =>
    apiFetch(`/clients/${id}/auth/credentials`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  activateMagicToken: (id: string): Promise<ClientMagicLinkDto> =>
    apiFetch(`/clients/${id}/auth/magic-token`, {
      method: 'POST',
    }),

  delete: (id: string): Promise<void> => apiFetch(`/clients/${id}`, { method: 'DELETE' }),
};
