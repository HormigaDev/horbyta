import { apiFetchClient } from 'src/boot/fetch';
import type { QueryPayload } from 'src/types';
import type {
  ClientLoginPayload,
  ClientLoginResponse,
  ClientMagicTokenLoginPayload,
} from 'src/types/clientAuth';

const authProbePayload: QueryPayload = {
  page: 1,
  limit: 1,
  order_by: null,
  order_asc: false,
  filters: [],
};

export const ClientAuthService = {
  login: (payload: ClientLoginPayload): Promise<ClientLoginResponse> =>
    apiFetchClient('/auth/login-client', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  loginWithMagicToken: (payload: ClientMagicTokenLoginPayload): Promise<ClientLoginResponse> =>
    apiFetchClient('/auth/login-client-token', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),

  checkSession: async (): Promise<void> => {
    const params = new URLSearchParams({ q: JSON.stringify(authProbePayload) });
    await apiFetchClient(`/projects/client-search?${params.toString()}`, {
      method: 'GET',
    });
  },
};
