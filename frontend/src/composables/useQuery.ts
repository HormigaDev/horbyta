import { ref, reactive, computed, type Ref } from 'vue';
import type {
  QueryPayload,
  FilterOperator,
  FilterSchema,
  PaginatedResponse,
  ColumnType,
  FilterFieldConfig,
} from '../types/query';

const resolveColumnType = (value: ColumnType | FilterFieldConfig): ColumnType => {
  if (typeof value === 'string') return value;
  return value.type;
};

export function useQuery<T>(
  fetcher: (payload: QueryPayload) => Promise<PaginatedResponse<T>>,
  schema: FilterSchema,
  defaultLimit = 10,
  defaultOrderBy: string | null = null,
  defaultOrderAsc = false,
) {
  const query = reactive<QueryPayload>({
    page: 1,
    limit: defaultLimit,
    order_by: defaultOrderBy,
    order_asc: defaultOrderAsc,
    filters: [],
  });

  const items: Ref<T[]> = ref([]);
  const total = ref(0);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const totalPages = computed(() => Math.ceil(total.value / query.limit));

  const fetchItems = async () => {
    isLoading.value = true;
    error.value = null;

    try {
      const payloadString = JSON.stringify(query);
      const res = await fetcher(JSON.parse(payloadString));

      items.value = res.items;
      total.value = res.total;
    } catch (err: unknown) {
      if (err instanceof Error) {
        error.value = err.message;
      } else {
        error.value = 'Error ejecutando la consulta.';
      }
      items.value = [];
      total.value = 0;
    } finally {
      isLoading.value = false;
    }
  };

  const setPage = (newPage: number) => {
    if (newPage < 1) return;
    query.page = newPage;
    void fetchItems();
  };

  const setLimit = (newLimit: number) => {
    if (newLimit < 1 || newLimit > 100) return;
    query.limit = newLimit;
    query.page = 1;
    void fetchItems();
  };

  const setOrder = (column: string) => {
    if (!schema[column]) {
      return;
    }

    if (query.order_by === column) {
      query.order_asc = !query.order_asc;
    } else {
      query.order_by = column;
      query.order_asc = true;
    }
    query.page = 1;
    void fetchItems();
  };

  const addFilter = (column: string, operator: FilterOperator, value: unknown) => {
    if (!schema[column]) {
      error.value = `Campo "${column}" no permitido.`;
      return;
    }
    if (query.filters.length >= 5) {
      error.value = 'Máximo 5 filtros.';
      return;
    }

    const colType = resolveColumnType(schema[column]);
    if ((operator === '><' || operator === '<>') && colType !== 'Text') {
      error.value = `Operador ${operator} no válido para ${colType}.`;
      return;
    }
    if (operator === '()' && colType !== 'Date' && colType !== 'Number') {
      error.value = `Operador () no válido para ${colType}.`;
      return;
    }

    query.filters.push({ column, operator, value });
    query.page = 1;
    void fetchItems();
  };

  const removeFilter = (index: number) => {
    if (index >= 0 && index < query.filters.length) {
      query.filters.splice(index, 1);
      query.page = 1;
      void fetchItems();
    }
  };

  const clearFilters = () => {
    query.filters = [];
    query.page = 1;
    void fetchItems();
  };

  return {
    query,
    items,
    total,
    totalPages,
    isLoading,
    error,

    fetch: fetchItems,
    setPage,
    setLimit,
    setOrder,
    addFilter,
    removeFilter,
    clearFilters,
  };
}
