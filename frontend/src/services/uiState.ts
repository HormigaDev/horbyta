import { apiFetch } from 'src/boot/fetch';
import type { UiStateEntryDto, UiStateListResponse, UiStateScope } from 'src/types/uiState';

const BASE_URL = '/users/me/ui-state';

const encode = (segment: string) => encodeURIComponent(segment);

export const UiStateService = {
  list: <T = unknown>(scope: UiStateScope): Promise<UiStateListResponse<T>> => {
    const params = new URLSearchParams({ scope });
    return apiFetch(`${BASE_URL}?${params.toString()}`, { method: 'GET' });
  },

  get: <T = unknown>(scope: UiStateScope, key: string): Promise<UiStateEntryDto<T>> =>
    apiFetch(`${BASE_URL}/${encode(scope)}/${encode(key)}`, { method: 'GET' }),

  upsert: <T = unknown>(
    scope: UiStateScope,
    key: string,
    value: T,
  ): Promise<UiStateEntryDto<T>> =>
    apiFetch(`${BASE_URL}/${encode(scope)}/${encode(key)}`, {
      method: 'PUT',
      body: JSON.stringify({ value }),
    }),

  batchUpsert: <T = unknown>(
    scope: UiStateScope,
    entries: Array<{ key: string; value: T }>,
  ): Promise<{ items: UiStateEntryDto<T>[] }> =>
    apiFetch(`${BASE_URL}/${encode(scope)}/batch`, {
      method: 'PUT',
      body: JSON.stringify({ entries }),
    }),

  remove: (scope: UiStateScope, key: string): Promise<{ message: string }> =>
    apiFetch(`${BASE_URL}/${encode(scope)}/${encode(key)}`, { method: 'DELETE' }),

  clearScope: (scope: UiStateScope): Promise<{ message: string }> =>
    apiFetch(`${BASE_URL}/${encode(scope)}`, { method: 'DELETE' }),
};
