import { defineStore } from 'pinia';
import { ref } from 'vue';
import { ClientAuthService } from 'src/services/clientAuth';
import type {
  ClientAuthIdentity,
  ClientLoginPayload,
  ClientLoginResponse,
  ClientMagicTokenLoginPayload,
} from 'src/types/clientAuth';

const PORTAL_USER_ID_STORAGE_KEY = 'client_portal_user_id';

const readStoredPortalUserId = (): string | null => {
  if (typeof window === 'undefined') return null;

  const value = window.localStorage.getItem(PORTAL_USER_ID_STORAGE_KEY)?.trim();
  return value && value.length > 0 ? value : null;
};

const writeStoredPortalUserId = (userId: string) => {
  if (typeof window === 'undefined') return;

  window.localStorage.setItem(PORTAL_USER_ID_STORAGE_KEY, userId);
};

export const useClientAuthStore = defineStore('clientAuth', () => {
  const isAuthenticated = ref(false);
  const isInitialized = ref(false);
  const client = ref<ClientAuthIdentity | null>(null);
  const portalUserId = ref<string | null>(readStoredPortalUserId());

  function setPortalUserId(userId: string) {
    const normalizedUserId = userId.trim();
    if (!normalizedUserId) return;

    portalUserId.value = normalizedUserId;
    writeStoredPortalUserId(normalizedUserId);
  }

  function setAuthenticated(status: boolean, clientData?: ClientAuthIdentity) {
    isAuthenticated.value = status;
    if (status && clientData) {
      client.value = clientData;
      setPortalUserId(clientData.user_id);
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
    portalUserId,
    setPortalUserId,
    setAuthenticated,
    clearAuth,
    checkAuth,
    login,
    loginWithMagicToken,
  };
});
