import { boot } from 'quasar/wrappers';
import { useAuthStore } from 'src/stores/auth';
import { useClientAuthStore } from 'src/stores/clientAuth';

async function request(url: string, options: RequestInit, scope: 'user' | 'client') {
  const headers = new Headers(options.headers);
  const isMultipartForm = options.body instanceof FormData;

  if (!isMultipartForm && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`/api/v1${url}`, {
    ...options,
    headers,
    credentials: 'include',
  });

  if (response.status === 401) {
    if (scope === 'client') {
      const clientAuthStore = useClientAuthStore();
      clientAuthStore.clearAuth();
      window.location.hash = '/client/login';
    } else {
      const authStore = useAuthStore();
      authStore.clearAuth();
      window.location.hash = '/login';
    }
  }

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new Error(errorData.error || `Error HTTP: ${response.status}`);
  }

  return response.json();
}

async function apiFetch(url: string, options: RequestInit = {}) {
  return request(url, options, 'user');
}

async function apiFetchClient(url: string, options: RequestInit = {}) {
  return request(url, options, 'client');
}

export default boot(({ app }) => {
  app.config.globalProperties.$api = apiFetch;
  app.config.globalProperties.$apiClient = apiFetchClient;
});

export { apiFetch };
export { apiFetchClient };
