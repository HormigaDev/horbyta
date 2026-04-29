import { apiFetchClient } from 'src/boot/fetch';
import type {
  ClientAuthIdentity,
  ClientLoginPayload,
  ClientLoginResponse,
  ClientMagicTokenLoginPayload,
} from 'src/types/clientAuth';

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

  checkSession: (): Promise<ClientAuthIdentity> =>
    apiFetchClient('/auth/client-me', { method: 'GET' }),
};
