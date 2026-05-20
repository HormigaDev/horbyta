import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UiStateService } from 'src/services/uiState';
import type { UiStateEntryDto, UiStateScope } from 'src/types/uiState';

const FLUSH_DEBOUNCE_MS = 400;

interface ScopeState {
  entries: Map<string, UiStateEntryDto>;
  loaded: boolean;
  loadingPromise: Promise<void> | null;
}

interface SetValueOptions {
  debounce?: boolean;
}

const buildScopeState = (): ScopeState => ({
  entries: new Map(),
  loaded: false,
  loadingPromise: null,
});

export const useUiStateStore = defineStore('uiState', () => {
  const scopes = ref<Map<UiStateScope, ScopeState>>(new Map());
  const pendingFlushes = new Map<string, ReturnType<typeof setTimeout>>();

  const getScopeState = (scope: UiStateScope): ScopeState => {
    let state = scopes.value.get(scope);
    if (!state) {
      state = buildScopeState();
      scopes.value.set(scope, state);
    }
    return state;
  };

  const ensureScopeLoaded = async (scope: UiStateScope): Promise<void> => {
    const state = getScopeState(scope);
    if (state.loaded) return;
    if (state.loadingPromise) {
      await state.loadingPromise;
      return;
    }

    state.loadingPromise = (async () => {
      try {
        const response = await UiStateService.list(scope);
        const next = new Map<string, UiStateEntryDto>();
        for (const item of response.items) {
          next.set(item.key, item);
        }
        state.entries = next;
        state.loaded = true;
      } finally {
        state.loadingPromise = null;
      }
    })();

    await state.loadingPromise;
  };

  const getValue = <T>(scope: UiStateScope, key: string, fallback: T): T => {
    const entry = getScopeState(scope).entries.get(key);
    return entry ? (entry.value as T) : fallback;
  };

  const flushKey = (scope: UiStateScope, key: string) => {
    const flushKeyId = `${scope}:${key}`;
    const existing = pendingFlushes.get(flushKeyId);
    if (existing) {
      clearTimeout(existing);
      pendingFlushes.delete(flushKeyId);
    }
  };

  const setValue = async <T>(
    scope: UiStateScope,
    key: string,
    value: T,
    options: SetValueOptions = {},
  ): Promise<void> => {
    const state = getScopeState(scope);
    const stamp = new Date().toISOString();
    state.entries.set(key, { scope, key, value, updated_at: stamp });

    flushKey(scope, key);

    if (options.debounce === false) {
      return UiStateService.upsert(scope, key, value).then((entry) => {
        state.entries.set(key, entry);
      });
    }

    const flushKeyId = `${scope}:${key}`;
    const timeout = setTimeout(() => {
      pendingFlushes.delete(flushKeyId);
      void UiStateService.upsert(scope, key, value).then((entry) => {
        state.entries.set(key, entry);
      });
    }, FLUSH_DEBOUNCE_MS);
    pendingFlushes.set(flushKeyId, timeout);

    return Promise.resolve();
  };

  const removeValue = async (scope: UiStateScope, key: string): Promise<void> => {
    const state = getScopeState(scope);
    state.entries.delete(key);
    flushKey(scope, key);
    await UiStateService.remove(scope, key);
  };

  const clearScope = async (scope: UiStateScope): Promise<void> => {
    const state = getScopeState(scope);
    state.entries.clear();
    state.loaded = true;
    for (const id of [...pendingFlushes.keys()]) {
      if (id.startsWith(`${scope}:`)) {
        const timeout = pendingFlushes.get(id);
        if (timeout) clearTimeout(timeout);
        pendingFlushes.delete(id);
      }
    }
    await UiStateService.clearScope(scope);
  };

  const listScope = <T>(scope: UiStateScope): UiStateEntryDto<T>[] => {
    return [...getScopeState(scope).entries.values()] as UiStateEntryDto<T>[];
  };

  const reset = () => {
    for (const timeout of pendingFlushes.values()) {
      clearTimeout(timeout);
    }
    pendingFlushes.clear();
    scopes.value = new Map();
  };

  return {
    ensureScopeLoaded,
    getValue,
    setValue,
    removeValue,
    clearScope,
    listScope,
    reset,
  };
});
