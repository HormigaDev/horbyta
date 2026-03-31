import { boot } from 'quasar/wrappers';
import { useAuthStore } from 'src/stores/auth';

async function apiFetch(url: string, options: RequestInit = {}) {
  const response = await fetch(`/api/v1${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (response.status === 401) {
    const authStore = useAuthStore();
    authStore.clearAuth();
    window.location.hash = '#/login';
  }

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new Error(errorData.error || `Error HTTP: ${response.status}`);
  }

  return response.json();
}

export default boot(({ app }) => {
  app.config.globalProperties.$api = apiFetch;
});

export { apiFetch };
