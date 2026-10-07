import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UiStateService } from 'src/services/uiState';
import { useClientAuthStore } from 'src/stores/clientAuth';
import type { Filter } from '../types/query';
export interface TableState {
  pagination?: { sortBy: string | null; descending: boolean; page: number; rowsPerPage: number };
  filters?: Filter[];
  visibleColumns?: string[];
}
const DATATABLE_SCOPE = 'datatable' as const;
export const useDataTableStore = defineStore('datatable', () => {
  const tables = ref<Record<string, TableState>>({});
  const queues = new Map<string, Promise<void>>();
  let generation = 0;
  const portalKey = (tableId: string): string | null => {
    if (!tableId.startsWith('client_')) return null;
    const identity = useClientAuthStore().client;
    if (!identity) throw new Error('Client identity is required for table preferences');
    return `horbyta:client-table:v1:${identity.user_id}:${identity.client_id}:${tableId}`;
  };
  const reset = () => {
    generation++;
    tables.value = {};
    queues.clear();
  };
  async function getTableState(tableId: string, defaultState?: TableState): Promise<TableState> {
    const current = generation;
    const key = portalKey(tableId);
    if (key) {
      let stored: TableState | undefined;
      try {
        const parsed = JSON.parse(localStorage.getItem(key) ?? 'null');
        if (
          parsed?.version === 1 &&
          parsed.value &&
          typeof parsed.value === 'object' &&
          !Array.isArray(parsed.value)
        )
          stored = parsed.value;
      } catch {
        /* Defaults are usable when optional browser storage fails. */
      }
      tables.value[tableId] = { ...defaultState, ...stored };
      return tables.value[tableId];
    }
    const response = await UiStateService.list<TableState>(DATATABLE_SCOPE);
    if (current !== generation) return {};
    const remote = response.items.find((item) => item.key === tableId)?.value;
    tables.value[tableId] = { ...defaultState, ...remote };
    return tables.value[tableId];
  }
  function saveTableState(tableId: string, state: Partial<TableState>): Promise<void> {
    const current = generation;
    const key = portalKey(tableId);
    const operation = (queues.get(tableId) ?? Promise.resolve())
      .catch(() => {})
      .then(async () => {
        if (current !== generation) return;
        const next = { ...tables.value[tableId], ...state };
        if (key) localStorage.setItem(key, JSON.stringify({ version: 1, value: next }));
        else await UiStateService.upsert<TableState>(DATATABLE_SCOPE, tableId, next);
        if (current === generation) tables.value[tableId] = next;
      });
    queues.set(tableId, operation);
    return operation;
  }
  const saveVisibleColumns = (tableId: string, visibleColumns: string[]) =>
    saveTableState(tableId, { visibleColumns: [...visibleColumns] });
  async function resetTableState(tableId: string) {
    await saveTableState(tableId, {});
    const current = generation;
    const key = portalKey(tableId);
    if (key) localStorage.removeItem(key);
    else await UiStateService.remove(DATATABLE_SCOPE, tableId);
    if (current === generation) delete tables.value[tableId];
  }
  return { tables, getTableState, saveTableState, saveVisibleColumns, resetTableState, reset };
});
