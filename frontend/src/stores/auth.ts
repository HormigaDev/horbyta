import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useAuthStore = defineStore('auth', () => {
  const isAuthenticated = ref(false);

  function setAuthenticated(status: boolean) {
    isAuthenticated.value = status;
  }

  function clearAuth() {
    isAuthenticated.value = false;
  }

  return { isAuthenticated, setAuthenticated, clearAuth };
});
