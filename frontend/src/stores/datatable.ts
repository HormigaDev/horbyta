import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
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

export const useDataTableStore = defineStore('datatable', () => {
  const tables = ref<Record<string, TableState>>({});

  const loadState = () => {
    const raw = localStorage.getItem('datatable_states');
    if (raw) {
      try {
        tables.value = JSON.parse(raw);
      } catch (e) {
        console.error('Error parseando estados de tablas:', e);
      }
    }
  };

  loadState();

  watch(
    tables,
    (newState) => {
      localStorage.setItem('datatable_states', JSON.stringify(newState));
    },
    { deep: true },
  );

  const getTableState = (tableId: string, defaultState?: TableState): TableState => {
    if (!tables.value[tableId] && defaultState) {
      tables.value[tableId] = { ...defaultState };
    }
    return tables.value[tableId] || {};
  };

  const saveTableState = (tableId: string, state: Partial<TableState>) => {
    tables.value[tableId] = {
      ...(tables.value[tableId] || {}),
      ...state,
    };
  };

  const resetTableState = (tableId: string) => {
    delete tables.value[tableId];
  };

  return {
    tables,
    getTableState,
    saveTableState,
    resetTableState,
  };
});
