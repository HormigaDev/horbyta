import { reactive, ref, watch } from 'vue';

interface UseChartCardStateOptions<TFilters extends object, TRow> {
  defaults: () => TFilters;
  fetchRows: (filters: TFilters) => Promise<TRow[]>;
  initialRows?: (() => TRow[]) | TRow[] | null;
  initialFilters?: TFilters | null;
}

export interface ChartCardState<TFilters extends object, TRow> {
  rows: { value: TRow[] };
  appliedFilters: TFilters;
  draftFilters: TFilters;
  isLoading: { value: boolean };
  loadError: { value: string | null };
  dialogOpen: { value: boolean };
  openDialog: () => void;
  applyDraft: () => Promise<void>;
  resetDraft: () => Promise<void>;
  refetch: () => Promise<void>;
  setRows: (next: TRow[]) => void;
}

export function useChartCardState<TFilters extends object, TRow>(
  options: UseChartCardStateOptions<TFilters, TRow>,
): ChartCardState<TFilters, TRow> {
  const initialRows = () =>
    typeof options.initialRows === 'function' ? options.initialRows() : options.initialRows;
  const rows = ref<TRow[]>(initialRows() ?? []) as { value: TRow[] };
  const appliedFilters = reactive<TFilters>(
    options.initialFilters ? { ...options.initialFilters } : options.defaults(),
  ) as TFilters;
  const draftFilters = reactive<TFilters>({ ...appliedFilters }) as TFilters;
  const isLoading = ref(false);
  const loadError = ref<string | null>(null);
  const dialogOpen = ref(false);

  const assign = (target: TFilters, source: TFilters) => {
    Object.keys(target).forEach((key) => {
      delete (target as Record<string, unknown>)[key];
    });
    Object.assign(target, source);
  };

  let requestId = 0;
  const fetchCandidate = async (candidate: TFilters) => {
    const id = ++requestId;
    isLoading.value = true;
    loadError.value = null;
    try {
      const data = await options.fetchRows({ ...candidate });
      if (id !== requestId) return false;
      rows.value = data;
      return true;
    } catch (error) {
      if (id === requestId)
        loadError.value = error instanceof Error ? error.message : 'Unknown error';
      throw error;
    } finally {
      if (id === requestId) isLoading.value = false;
    }
  };
  const refetch = async () => {
    await fetchCandidate({ ...appliedFilters });
  };

  const openDialog = () => {
    assign(draftFilters, { ...appliedFilters });
    dialogOpen.value = true;
  };

  const applyDraft = async () => {
    const candidate = { ...draftFilters };
    if (await fetchCandidate(candidate)) {
      assign(appliedFilters, candidate);
      dialogOpen.value = false;
    }
  };

  const resetDraft = async () => {
    const fresh = options.defaults();
    if (await fetchCandidate(fresh)) {
      assign(draftFilters, fresh);
      assign(appliedFilters, fresh);
      dialogOpen.value = false;
    }
  };

  const setRows = (next: TRow[]) => {
    rows.value = next;
  };

  watch(initialRows, (next) => {
    if (next) {
      if (JSON.stringify(appliedFilters) === JSON.stringify(options.defaults())) rows.value = next;
      else void refetch().catch(() => {});
    }
  });

  return {
    rows,
    appliedFilters,
    draftFilters,
    isLoading,
    loadError,
    dialogOpen,
    openDialog,
    applyDraft,
    resetDraft,
    refetch,
    setRows,
  };
}
