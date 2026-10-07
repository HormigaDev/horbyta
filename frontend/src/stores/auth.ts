import { defineStore } from 'pinia';
import type { User } from 'src/types';
import { ref } from 'vue';
import { useThemeStore } from 'src/stores/theme';
import { useDataTableStore } from 'src/stores/datatable';
import { apiFetch } from 'src/boot/fetch';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useUiStateStore } from 'src/stores/uiState';

export const useAuthStore = defineStore('auth', () => {
  const isAuthenticated = ref(false);
  const isInitialized = ref(false);
  const user = ref<User | null>(null);

  function setAuthenticated(status: boolean, userData?: User) {
    isAuthenticated.value = status;
    if (userData) {
      user.value = userData;
      useThemeStore().bindAccount(userData.id);
    }
  }

  function clearAuth() {
    const userPreferencesStore = useUserPreferencesStore();
    const uiStateStore = useUiStateStore();

    isAuthenticated.value = false;
    user.value = null;
    userPreferencesStore.resetToDefaults();
    useThemeStore().bindAccount(null);
    useDataTableStore().reset();
    uiStateStore.reset();
  }

  async function checkAuth() {
    if (isInitialized.value) return;
    try {
      const response = await apiFetch('/health');
      setAuthenticated(true, response.user);
    } catch {
      clearAuth();
    } finally {
      isInitialized.value = true;
    }
  }

  return { isAuthenticated, isInitialized, user, setAuthenticated, clearAuth, checkAuth };
});
