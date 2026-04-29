import { defineStore } from 'pinia';
import { ref } from 'vue';
import { ClientAuthService } from 'src/services/clientAuth';
import type {
  ClientAuthIdentity,
  ClientLoginPayload,
  ClientLoginResponse,
  ClientMagicTokenLoginPayload,
} from 'src/types/clientAuth';

export const useClientAuthStore = defineStore('clientAuth', () => {
  const isAuthenticated = ref(false);
  const isInitialized = ref(false);
  const client = ref<ClientAuthIdentity | null>(null);

  function setAuthenticated(status: boolean, clientData?: ClientAuthIdentity) {
    isAuthenticated.value = status;
    if (status && clientData) {
      client.value = clientData;
    } else if (!status) {
      client.value = null;
    }
  }

  function clearAuth() {
    isAuthenticated.value = false;
    client.value = null;
  }

  async function checkAuth() {
    if (isInitialized.value) return;
    try {
      const clientData = await ClientAuthService.checkSession();
      setAuthenticated(true, clientData);
    } catch {
      clearAuth();
    } finally {
      isInitialized.value = true;
    }
  }

  async function login(payload: ClientLoginPayload): Promise<ClientLoginResponse> {
    const response = await ClientAuthService.login(payload);
    setAuthenticated(true, response.client);
    isInitialized.value = true;
    return response;
  }

  async function loginWithMagicToken(
    payload: ClientMagicTokenLoginPayload,
  ): Promise<ClientLoginResponse> {
    const response = await ClientAuthService.loginWithMagicToken(payload);
    setAuthenticated(true, response.client);
    isInitialized.value = true;
    return response;
  }

  return {
    isAuthenticated,
    isInitialized,
    client,
    setAuthenticated,
    clearAuth,
    checkAuth,
    login,
    loginWithMagicToken,
  };
});
