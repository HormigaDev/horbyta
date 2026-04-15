import { ref } from 'vue';
import type { Filter, PaginatedResponse, QueryPayload } from 'src/types';

export interface SelectByTermOption {
  label: string;
  value: string;
  color?: string;
  [key: string]: unknown;
}

interface UseSelectByTermConfig<T> {
  termColumn: string;
  toOption: (item: T) => SelectByTermOption;
  orderBy?: string | null;
  orderAsc?: boolean;
}

export function useSelectByTerm<T>(
  searchFn: (payload: QueryPayload) => Promise<PaginatedResponse<T>>,
  config: UseSelectByTermConfig<T>,
) {
  const options = ref<SelectByTermOption[]>([]);
  const isLoading = ref(false);

  const setOptions = (newOptions: SelectByTermOption[]) => {
    options.value = newOptions;
  };

  const load = async (term = '', extraFilters: Filter[] = []) => {
    isLoading.value = true;
    try {
      const filters: Filter[] = [...extraFilters];
      const cleanTerm = term.trim();

      if (cleanTerm.length > 0) {
        filters.push({
          column: config.termColumn,
          operator: '><',
          value: cleanTerm,
        });
      }

      const payload: QueryPayload = {
        page: 1,
        limit: 10,
        order_by: config.orderBy ?? null,
        order_asc: config.orderAsc ?? true,
        filters,
      };

      const response = await searchFn(payload);
      options.value = response.items.map(config.toOption);
    } finally {
      isLoading.value = false;
    }
  };

  return {
    options,
    isLoading,
    load,
    setOptions,
  };
}
