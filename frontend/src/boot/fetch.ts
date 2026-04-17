import { boot } from 'quasar/wrappers';
import { useAuthStore } from 'src/stores/auth';
import { useClientAuthStore } from 'src/stores/clientAuth';

interface ApiHttpErrorContext {
  status: number;
  method: string;
  url: string;
  payload: Record<string, unknown>;
}

export class ApiHttpError extends Error {
  readonly status: number;
  readonly method: string;
  readonly url: string;
  readonly type: string | null;
  readonly backendError: string | null;
  readonly payload: Record<string, unknown>;

  constructor(context: ApiHttpErrorContext) {
    const backendError =
      typeof context.payload.error === 'string' ? context.payload.error.trim() : null;

    super(backendError && backendError.length > 0 ? backendError : `http error ${context.status}`);

    this.name = 'ApiHttpError';
    this.status = context.status;
    this.method = context.method;
    this.url = context.url;
    this.type = typeof context.payload.type === 'string' ? context.payload.type : null;
    this.backendError = backendError;
    this.payload = context.payload;
  }

  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      status: this.status,
      method: this.method,
      url: this.url,
      type: this.type,
      error: this.backendError,
      payload: this.payload,
    };
  }
}

export const isApiHttpError = (error: unknown): error is ApiHttpError =>
  error instanceof ApiHttpError;

const toRecord = (value: unknown): Record<string, unknown> => {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return {};
};

async function parseErrorPayload(response: Response): Promise<Record<string, unknown>> {
  const rawJson = await response
    .clone()
    .json()
    .catch(() => null);

  if (rawJson !== null) {
    return toRecord(rawJson);
  }

  const fallbackText = await response.text().catch(() => '');
  if (fallbackText.trim().length > 0) {
    return { error: fallbackText.trim() };
  }

  return {};
}

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
    const errorPayload = await parseErrorPayload(response);
    throw new ApiHttpError({
      status: response.status,
      method: (options.method ?? 'GET').toUpperCase(),
      url,
      payload: errorPayload,
    });
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
