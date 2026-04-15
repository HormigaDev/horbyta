import { defineStore } from 'pinia';
import type { User } from 'src/types';
import { ref } from 'vue';
import { apiFetch } from 'src/boot/fetch';
import { useUserPreferencesStore } from 'src/stores/userPreferences';

export const useAuthStore = defineStore('auth', () => {
  const isAuthenticated = ref(false);
  const isInitialized = ref(false);
  const user = ref<User | null>(null);

  function setAuthenticated(status: boolean, userData?: User) {
    isAuthenticated.value = status;
    if (userData) {
      user.value = userData;
    }
  }

  function clearAuth() {
    const userPreferencesStore = useUserPreferencesStore();

    isAuthenticated.value = false;
    user.value = null;
    userPreferencesStore.resetToDefaults();
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
