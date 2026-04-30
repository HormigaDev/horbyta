import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UiStateService } from 'src/services/uiState';
import type { Filter } from '../types/query';

export interface TableState {
  pagination?: {
    sortBy: string | null;
    descending: boolean;
    page: number;
    rowsPerPage: number;
  };
  filters?: Filter[];
  visibleColumns?: string[];
}

const DATATABLE_SCOPE = 'datatable' as const;

export const useDataTableStore = defineStore('datatable', () => {
  const tables = ref<Record<string, TableState>>({});

  const getTableState = async (
    tableId: string,
    defaultState?: TableState,
  ): Promise<TableState> => {
    if (defaultState && !tables.value[tableId]) {
      tables.value[tableId] = { ...defaultState };
    }

    const response = await UiStateService.list<TableState>(DATATABLE_SCOPE);
    const remoteState = response.items.find((item) => item.key === tableId)?.value;
    if (remoteState) {
      tables.value[tableId] = { ...remoteState };
    }

    return tables.value[tableId] || {};
  };

  const saveTableState = async (
    tableId: string,
    state: Partial<TableState>,
  ): Promise<void> => {
    tables.value[tableId] = {
      ...(tables.value[tableId] || {}),
      ...state,
    };

    await UiStateService.upsert<TableState>(DATATABLE_SCOPE, tableId, tables.value[tableId]);
  };

  const saveVisibleColumns = async (tableId: string, visibleColumns: string[]): Promise<void> => {
    const nextState: TableState = {
      ...(tables.value[tableId] || {}),
      visibleColumns: [...visibleColumns],
    };

    tables.value[tableId] = nextState;

    await UiStateService.upsert<TableState>(DATATABLE_SCOPE, tableId, nextState);
  };

  const resetTableState = async (tableId: string) => {
    delete tables.value[tableId];
    await UiStateService.remove(DATATABLE_SCOPE, tableId);
  };

  return {
    tables,
    getTableState,
    saveTableState,
    saveVisibleColumns,
    resetTableState,
  };
});
