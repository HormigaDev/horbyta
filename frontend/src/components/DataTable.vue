<template>
  <q-card class="data-table-wrapper" flat bordered>
    <q-table
      separator="horizontal"
      :title="title"
      :rows="loading ? [] : rows"
      :columns="effectiveColumns"
      :row-key="rowKey"
      v-model:pagination="pagination"
      v-model:selected="selected"
      :selection="selectionMode"
      :loading="loading"
      :hide-bottom="loading"
      :rows-per-page-options="[5, 10, 20, 50, 100]"
      table-class="dt-fixed-layout"
      :visible-columns="visibleColumns"
      :dense="themeStore.appearance.density === 'compact' && !$q.screen.lt.sm"
      :rows-per-page-label="t('datatable.rowsPerPage')"
      :no-results-label="t('datatable.noResults')"
      :loading-label="t('datatable.loading')"
      :selected-rows-label="
        (numberOfRows: number) => {
          if (numberOfRows <= 0) return '';

          return numberOfRows > 1
            ? t('datatable.selectedRows.multiple', { count: numberOfRows })
            : t('datatable.selectedRows.single');
        }
      "
      @request="onRequest"
      @row-dblclick="onRowDblClick"
      class="custom-table dt-table"
      flat
      square
    >
      <template #loading
        ><span class="sr-only">{{ t('datatable.loading') }}</span></template
      >
      <template #top="_props">
        <div class="row full-width items-center q-pb-sm">
          <div class="col-xs-12 col-sm text-h5 text-weight-bold flex items-center" v-if="title">
            <q-icon v-if="icon" :name="icon" size="md" class="q-mr-sm" />

            {{ translateLabel(title) }}
            <q-badge color="primary" class="q-ml-sm" v-if="totalItems > 0">
              {{ totalItems }}
            </q-badge>
          </div>

          <div class="col-xs-12 col-sm-auto q-gutter-x-sm row justify-end items-center dt-actions">
            <slot name="top-actions"></slot>

            <q-btn
              v-if="!hideRefresh"
              flat
              round
              dense
              icon="refresh"
              style="color: var(--text-muted)"
              @click="emit('refresh')"
              :aria-label="t('datatable.tooltip.refresh')"
            >
              <q-tooltip>{{ t('datatable.tooltip.refresh') }}</q-tooltip>
            </q-btn>

            <q-btn
              flat
              round
              dense
              icon="filter_alt"
              style="color: var(--text-muted)"
              @click="showFilters = !showFilters"
              :aria-label="t('datatable.tooltip.filters')"
              :aria-expanded="showFilters"
              :style="appliedFilters.length > 0 ? 'color: var(--text-accent)' : ''"
            >
              <q-tooltip>{{ t('datatable.tooltip.filters') }}</q-tooltip>
              <q-badge
                v-if="appliedFilters.length > 0"
                color="primary"
                floating
                class="dt-filter-count"
                >{{ appliedFilters.length }}</q-badge
              >
            </q-btn>

            <q-btn
              flat
              round
              dense
              icon="view_column"
              style="color: var(--text-muted)"
              @click="openColumnDialog"
              :aria-label="t('datatable.tooltip.columns')"
            >
              <q-tooltip>{{ t('datatable.tooltip.columns') }}</q-tooltip>
            </q-btn>

            <q-separator vertical class="q-mx-sm" v-if="selected.length > 0" />

            <div class="row q-gutter-x-sm" v-if="selected.length > 0">
              <q-btn
                v-if="!hideView && selected.length === 1"
                outline
                color="secondary"
                icon="visibility"
                :label="t('datatable.actions.view')"
                @click="onView(selected[0] as T)"
              />
              <q-btn
                v-if="!hideEdit && selected.length === 1"
                outline
                color="primary"
                icon="edit"
                :label="t('datatable.actions.edit')"
                @click="onEdit"
              />
              <q-btn
                v-if="!hideDelete"
                outline
                color="negative"
                icon="delete"
                :label="t('datatable.actions.delete')"
                @click="onDelete"
              />
            </div>
            <div>
              <q-btn
                v-if="!hideCreate"
                unelevated
                color="primary"
                icon="add"
                :label="t('datatable.actions.new')"
                @click="onCreate"
              />
            </div>
          </div>
        </div>

        <div v-if="filterDraftChanged" class="text-caption text-secondary full-width" role="status">
          {{ t('ui.draftFilters') }}
        </div>
        <div v-if="preferenceError" class="full-width preference-error q-pa-sm" role="alert">
          {{ t('ui.preferenceError') }}
          <q-btn flat dense :label="t('ui.retry')" @click="retryPreferences" />
        </div>
        <q-slide-transition>
          <div
            v-show="showFilters"
            class="q-mt-sm q-pa-sm rounded-borders full-width"
            style="
              border: 1px solid var(--border-color);
              background-color: var(--card-bg) !important;
            "
          >
            <div
              v-for="(filter, index) in filters"
              :key="index"
              class="row q-col-gutter-sm q-mb-sm items-center"
            >
              <div class="col-12 col-sm-3">
                <q-select
                  v-model="filter.column"
                  :options="filterColumnOptions"
                  :label="t('datatable.filter.column')"
                  dense
                  outlined
                  emit-value
                  map-options
                  @update:model-value="onFilterColumnChange(filter)"
                />
              </div>
              <div class="col-12 col-sm-3">
                <q-select
                  v-model="filter.operator"
                  :options="getFilterOperators(filter.column)"
                  :label="t('datatable.filter.operator')"
                  dense
                  outlined
                  emit-value
                  map-options
                  :disable="!filter.column"
                />
              </div>
              <div class="col-12 col-sm-5">
                <slot
                  v-if="shouldUseSelectByTerm(filter)"
                  name="filter-value-select-by-term"
                  :filter="filter"
                  :index="index"
                  :column-key="filter.column"
                  :filters="filters"
                  :set-value="(value: unknown) => (filter.value = value)"
                >
                  <q-input
                    :model-value="toInputModelValue(filter.value)"
                    :label="t('datatable.filter.value')"
                    dense
                    outlined
                    :type="getFilterInputType(filter.column)"
                    :disable="!filter.operator"
                    @update:model-value="(value) => onInputFilterValue(filter, value)"
                  />
                </slot>
                <q-input
                  v-else
                  :model-value="toInputModelValue(filter.value)"
                  :label="t('datatable.filter.value')"
                  dense
                  outlined
                  :type="getFilterInputType(filter.column)"
                  :disable="!filter.operator"
                  @update:model-value="(value) => onInputFilterValue(filter, value)"
                />
              </div>
              <div class="col-xs-12 col-sm-1 flex justify-end">
                <q-btn
                  flat
                  round
                  color="negative"
                  icon="close"
                  :aria-label="t('datatable.filter.remove')"
                  size="sm"
                  @click="removeFilter(index)"
                >
                  <q-tooltip>{{ t('datatable.filter.remove') }}</q-tooltip>
                </q-btn>
              </div>
            </div>

            <div class="row justify-between q-mt-sm">
              <q-btn
                outline
                color="primary"
                icon="add"
                :label="t('datatable.filter.add')"
                size="sm"
                @click="addFilter"
                :disable="filters.length >= 5"
              />
              <div class="row q-gutter-x-sm">
                <q-btn
                  flat
                  color="negative"
                  :label="t('datatable.filter.clearAll')"
                  size="sm"
                  @click="clearFilters"
                  v-if="filters.length > 0"
                />
                <q-btn
                  unelevated
                  color="primary"
                  :label="t('datatable.filter.apply')"
                  size="sm"
                  @click="applyFilters"
                />
              </div>
            </div>
          </div>
        </q-slide-transition>

        <div v-if="error && rows.length" class="full-width preference-error q-pa-sm" role="alert">
          {{ t('ui.loadError') }} <q-btn flat :label="t('ui.retry')" @click="emit('refresh')" />
        </div>
        <slot name="filters"></slot>
      </template>

      <template #header="props">
        <q-tr :props="props">
          <q-th auto-width class="dt-selection-cell" v-if="selectionMode !== 'none'">
            <q-checkbox
              v-model="props.selected"
              :aria-label="t('datatable.selectedRows.multiple', { count: rows.length })"
            />
          </q-th>
          <q-th
            v-for="col in props.cols"
            :key="col.name"
            :props="props"
            class="text-weight-medium"
            :tabindex="col.sortable ? 0 : undefined"
            :aria-sort="
              col.sortable
                ? pagination.sortBy === col.name
                  ? pagination.descending
                    ? 'descending'
                    : 'ascending'
                  : 'none'
                : undefined
            "
            @keydown.enter.prevent="col.sortable && props.sort(col)"
            @keydown.space.prevent="col.sortable && props.sort(col)"
          >
            {{ translateLabel(col.label) }}
          </q-th>
        </q-tr>
      </template>

      <template #top-row="table">
        <q-tr v-if="loading" class="table-loading-row">
          <q-td :colspan="table.cols.length + (selectionMode === 'none' ? 0 : 1)">
            <ContentSkeleton variant="table" />
          </q-td>
        </q-tr>
      </template>

      <template #body-selection="cell"
        ><q-checkbox
          v-model="cell.selected"
          :aria-label="t('datatable.selectedRows.single') + ' ' + getRowId(cell.row)"
      /></template>
      <template #body-cell-open="cell">
        <q-td :props="cell" auto-width
          ><q-btn
            flat
            round
            dense
            icon="arrow_outward"
            :aria-label="t('ui.open')"
            @click.stop="onView(cell.row)"
            ><q-tooltip>{{ t('datatable.actions.view') }}</q-tooltip></q-btn
          ></q-td
        >
      </template>
      <template #body-cell="cell">
        <q-td :props="cell">
          <span
            v-if="cell.col.name === 'status'"
            class="status-badge"
            :data-status="getRowValue(cell.row, 'status')"
            >{{ cell.value }}</span
          >
          <button
            v-else-if="typeof cell.value === 'string' && cell.value.length > 40"
            type="button"
            class="cell-text"
            :aria-label="t('ui.showText') + ': ' + cell.value"
          >
            {{ cell.value
            }}<q-popup-proxy
              ><div class="cell-full-text q-pa-md">{{ cell.value }}</div></q-popup-proxy
            >
          </button>
          <span v-else>{{ cell.value }}</span>
        </q-td>
      </template>
      <template #no-data>
        <EmptyState
          v-if="!loading"
          class="full-width"
          :error="!!error"
          :title="
            error
              ? t('ui.loadError')
              : appliedFilters.length
                ? t('ui.noResults')
                : t('ui.emptyTitle')
          "
          :hint="
            error ? undefined : appliedFilters.length ? t('ui.noResultsHint') : t('ui.emptyHint')
          "
        >
          <q-btn
            v-if="error"
            outline
            color="primary"
            :label="t('ui.retry')"
            @click="emit('refresh')"
          />
          <q-btn
            v-else-if="appliedFilters.length"
            outline
            color="primary"
            :label="t('datatable.filter.clearAll')"
            @click="clearFilters"
          />
          <q-btn
            v-else-if="!hideCreate"
            unelevated
            color="primary"
            icon="add"
            :label="t('datatable.actions.new')"
            @click="onCreate"
          />
        </EmptyState>
      </template>

      <template v-for="(_, slotName) in $slots" #[slotName]="slotProps" :key="slotName">
        <slot :name="slotName" v-bind="slotProps || {}" />
      </template>
    </q-table>

    <q-dialog v-model="showColumnDialog">
      <q-card flat bordered class="dt-columns-dialog">
        <q-card-section class="row items-center justify-between q-pb-sm">
          <div class="text-h6">{{ t('datatable.columnsDialogTitle') }}</div>
          <q-btn flat round dense icon="close" :aria-label="t('common.close')" v-close-popup>
            <q-tooltip>{{ t('common.close') }}</q-tooltip>
          </q-btn>
        </q-card-section>

        <q-separator />

        <q-card-section class="dt-columns-list">
          <q-list separator>
            <q-item
              v-for="column in columnOptions"
              :key="column.value"
              tag="label"
              clickable
              v-ripple
            >
              <q-item-section avatar>
                <q-checkbox
                  v-model="draftVisibleColumns"
                  :aria-label="column.label"
                  :val="column.value"
                  :disable="
                    draftVisibleColumns.length === 1 && draftVisibleColumns.includes(column.value)
                  "
                />
              </q-item-section>
              <q-item-section>
                <q-item-label>{{ column.label }}</q-item-label>
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" v-close-popup />
          <q-btn
            unelevated
            color="primary"
            icon="save"
            :label="t('common.save')"
            :loading="isSavingColumnPreferences"
            :disable="draftVisibleColumns.length === 0"
            @click="saveColumnPreferences"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-card>
</template>

<script setup lang="ts" generic="T extends object">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import EmptyState from 'src/components/EmptyState.vue';
import { useThemeStore } from 'src/stores/theme';
import { ref, onMounted, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useDataTableStore } from '../stores/datatable';
import type { ColumnType, Filter, FilterOperator, FilterSchema } from '../types/query';
import { useQuasar, type QTableProps } from 'quasar';

type SelectionMode = 'none' | 'single' | 'multiple';

type LocalFilterFieldConfig = {
  type: ColumnType;
  label?: string;
  useSelectByTerm?: boolean;
};

interface Props {
  tableId: string;
  title?: string;
  icon?: string;
  rows?: T[];
  columns: NonNullable<QTableProps['columns']>;
  totalItems?: number;
  loading?: boolean;
  error?: string | null;
  rowKey?: string;
  selectionMode?: SelectionMode;
  hideCreate?: boolean;
  hideView?: boolean;
  hideEdit?: boolean;
  hideDelete?: boolean;
  hideRefresh?: boolean;
  filterSchema?: FilterSchema;
  initialFilters?: Filter[];
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  rows: () => [] as unknown as T[],
  totalItems: 0,
  loading: false,
  rowKey: 'id',
  selectionMode: 'multiple',
  hideCreate: false,
  hideView: false,
  hideEdit: false,
  hideDelete: false,
  hideRefresh: false,
  filterSchema: () => ({}),
  initialFilters: () => [],
});

const { t } = useI18n();
const $q = useQuasar();

const emit = defineEmits<{
  (
    e: 'request',
    requestProps: {
      pagination: { page: number; rowsPerPage: number; sortBy: string | null; descending: boolean };
      filters: Filter[];
      force?: boolean;
    },
  ): void;
  (e: 'create'): void;
  (e: 'view', id: string): void;
  (e: 'edit', ids: string[]): void;
  (e: 'delete', ids: string[]): void;
  (e: 'refresh'): void;
  (e: 'selection-change', selected: T[]): void;
  (e: 'sort', val: { sortBy: string | null; descending: boolean }): void;
}>();

const store = useDataTableStore();
const themeStore = useThemeStore();
const preferenceError = ref(false);
const appliedFilters = ref<Filter[]>([]);
const effectiveColumns = computed(() =>
  props.hideView
    ? props.columns
    : [
        {
          name: 'open',
          label: '',
          field: props.rowKey,
          align: 'left' as const,
          required: true,
          style: 'width: 56px',
          headerStyle: 'width: 56px',
        },
        ...props.columns,
      ],
);
const filterDraftChanged = computed(
  () => JSON.stringify(getValidFilters()) !== JSON.stringify(appliedFilters.value),
);
async function persistState() {
  try {
    await store.saveTableState(props.tableId, {
      pagination: { ...pagination.value },
      visibleColumns: [...visibleColumns.value],
      filters: JSON.parse(JSON.stringify(appliedFilters.value)) as Filter[],
    });
    preferenceError.value = false;
  } catch {
    preferenceError.value = true;
  }
}
const retryPreferences = () => {
  void persistState();
};
const selected = ref<T[]>([]);
const visibleColumns = ref(props.columns.map((c) => c.name));
const draftVisibleColumns = ref<string[]>([]);
const showColumnDialog = ref(false);
const isSavingColumnPreferences = ref(false);

const showFilters = ref(false);
const filters = ref<Partial<Filter>[]>([]);

const clearSelection = () => {
  if (selected.value.length > 0) {
    selected.value = [];
  }
};

const columnOptions = computed(() =>
  props.columns
    .filter((column) => column.label !== '')
    .map((column) => ({
      label: translateLabel(column.label),
      value: column.name,
    })),
);

const filterColumnOptions = computed(() => {
  return Object.keys(props.filterSchema).map((key) => {
    const col = props.columns.find(
      (c) => c.name === key || (typeof c.field === 'string' && c.field === key),
    );
    const config = getFilterConfig(key);

    const label = config?.label || (col ? String(col.label) : key);

    return {
      label: translateLabel(label),
      value: key,
    };
  });
});

const translateLabel = (label: unknown): string => {
  const asString = String(label);
  return asString.includes('.') ? t(asString) : asString;
};

const openColumnDialog = () => {
  draftVisibleColumns.value = [...visibleColumns.value];
  showColumnDialog.value = true;
};

const saveColumnPreferences = async () => {
  if (draftVisibleColumns.value.length === 0 || isSavingColumnPreferences.value) return;

  isSavingColumnPreferences.value = true;

  try {
    const nextVisibleColumns = [...draftVisibleColumns.value];
    await store.saveVisibleColumns(props.tableId, nextVisibleColumns);
    visibleColumns.value = nextVisibleColumns;
    preferenceError.value = false;
    showColumnDialog.value = false;
  } catch {
    preferenceError.value = true;
  } finally {
    isSavingColumnPreferences.value = false;
  }
};

const isFilterFieldConfig = (value: unknown): value is LocalFilterFieldConfig => {
  if (typeof value !== 'object' || value === null) return false;
  const maybeType = (value as { type?: unknown }).type;

  return (
    maybeType === 'Text' ||
    maybeType === 'Number' ||
    maybeType === 'Boolean' ||
    maybeType === 'Date'
  );
};

const getFilterConfig = (columnKey?: string): LocalFilterFieldConfig | null => {
  if (!columnKey) return null;

  const field = props.filterSchema[columnKey];
  if (!field) return null;

  if (typeof field === 'string') {
    return { type: field };
  }

  if (!isFilterFieldConfig(field)) {
    return null;
  }

  return field;
};

const getFilterType = (columnKey?: string): ColumnType | null => {
  const config = getFilterConfig(columnKey);
  if (!config) return null;
  return config.type;
};

const getFilterOperators = (columnKey?: string) => {
  if (!columnKey) return [];
  const config = getFilterConfig(columnKey);
  const type = config?.type;

  if (!type) return [];

  if (config.useSelectByTerm) {
    return [
      { label: t('datatable.operator.eq'), value: '=' },
      { label: t('datatable.operator.neq'), value: '!=' },
    ];
  }

  const options = [
    { label: t('datatable.operator.eq'), value: '=' },
    { label: t('datatable.operator.neq'), value: '!=' },
  ];

  if (type === 'Number' || type === 'Date') {
    options.push(
      { label: t('datatable.operator.gt'), value: '>' },
      { label: t('datatable.operator.lt'), value: '<' },
      { label: t('datatable.operator.gte'), value: '>=' },
      { label: t('datatable.operator.lte'), value: '<=' },
    );
  }

  if (type === 'Text') {
    options.push(
      { label: t('datatable.operator.contains'), value: '><' },
      { label: t('datatable.operator.notContains'), value: '<>' },
    );
  }

  if (type === 'Date') {
    options.push({ label: t('datatable.operator.between'), value: '()' });
  }

  return options;
};

const getFilterInputType = (columnKey?: string) => {
  if (!columnKey) return 'text';
  const type = getFilterType(columnKey);
  if (type === 'Number') return 'number';
  if (type === 'Date') return 'date';
  return 'text';
};

const shouldUseSelectByTerm = (filter: Partial<Filter>) => {
  if (!filter.column || !filter.operator) return false;

  const config = getFilterConfig(filter.column);
  if (!config?.useSelectByTerm) return false;

  return filter.operator === '=' || filter.operator === '!=';
};

const toInputModelValue = (value: unknown): string | number | null => {
  if (typeof value === 'string' || typeof value === 'number') return value;
  if (value === null || value === undefined) return null;
  return null;
};

const onInputFilterValue = (filter: Partial<Filter>, value: string | number | null) => {
  filter.value = value;
};

const addFilter = () => {
  if (filters.value.length < 5) {
    const defaultOperator = '=' as FilterOperator;
    filters.value.push({ column: '', operator: defaultOperator, value: '' });
  }
};

const removeFilter = (index: number) => {
  filters.value.splice(index, 1);
};

const clearFilters = () => {
  filters.value = [];
  applyFilters();
};

const getValidFilters = (): Filter[] => {
  return filters.value.filter(
    (f) => f.column && f.operator && f.value !== '' && f.value !== null && f.value !== undefined,
  ) as Filter[];
};

const applyFilters = () => {
  const validFilters = getValidFilters();
  onRequest({
    pagination: {
      page: 1,
      rowsPerPage: pagination.value.rowsPerPage,
      sortBy: pagination.value.sortBy,
      descending: pagination.value.descending,
    },
    filters: validFilters,
    force: true,
  });
};

const onFilterColumnChange = (filter: Partial<Filter>) => {
  filter.operator = '=' as FilterOperator;
  filter.value = '';
};

const defaultPagination = {
  sortBy: 'created_at' as string | null,
  descending: true,
  page: 1,
  rowsPerPage: 10,
  rowsNumber: props.totalItems,
};

const pagination = ref({ ...defaultPagination });

onMounted(async () => {
  let savedState: Awaited<ReturnType<typeof store.getTableState>> = {};
  try {
    savedState = await store.getTableState(props.tableId);
  } catch {
    preferenceError.value = true;
  }

  if (savedState.pagination) {
    pagination.value = {
      ...defaultPagination,
      ...savedState.pagination,
      sortBy: savedState.pagination.sortBy || defaultPagination.sortBy,
      rowsNumber: props.totalItems,
    };
  }
  if (savedState.visibleColumns && savedState.visibleColumns.length > 0) {
    visibleColumns.value = savedState.visibleColumns.filter((name) =>
      props.columns.some((c) => c.name === name),
    );
  }

  if (props.initialFilters.length > 0) {
    filters.value = props.initialFilters.map((f) => ({ ...f })) as Partial<Filter>[];
    showFilters.value = true;
    pagination.value.page = 1;
  } else if (savedState.filters) {
    filters.value = savedState.filters.map((f) => ({ ...f })) as Partial<Filter>[];
  }

  onRequest({
    pagination: {
      page: pagination.value.page,
      rowsPerPage: pagination.value.rowsPerPage,
      sortBy: pagination.value.sortBy,
      descending: pagination.value.descending,
    },
    filters: getValidFilters(),
  });
});

watch(
  () => props.totalItems,
  (newTotal) => {
    pagination.value.rowsNumber = newTotal;
  },
);

watch(selected, (newSel) => {
  emit('selection-change', newSel as unknown as T[]);
});

const onRequest = (requestProps: {
  pagination: { page: number; rowsPerPage: number; sortBy: string | null; descending: boolean };
  filters?: Filter[];
  force?: boolean;
}) => {
  const { page, rowsPerPage, sortBy, descending } = requestProps.pagination;
  const requestFilters = requestProps.filters ?? appliedFilters.value;

  const sortChanged =
    pagination.value.sortBy !== sortBy || pagination.value.descending !== descending;

  pagination.value.page = page;
  pagination.value.rowsPerPage = rowsPerPage;
  pagination.value.sortBy = sortBy;
  pagination.value.descending = descending;

  appliedFilters.value = requestFilters.map((filter) => ({ ...filter }));
  void persistState();

  emit('request', {
    pagination: { page, rowsPerPage, sortBy, descending },
    filters: requestFilters,
    ...(requestProps.force ? { force: true } : {}),
  });

  if (sortChanged) {
    emit('sort', { sortBy, descending });
  }
};

const getRowValue = (row: object, key: string): unknown => {
  return (row as Record<string, unknown>)[key];
};

const getRowId = (row: object): string => {
  return String(getRowValue(row, props.rowKey));
};

watch(
  () => props.rows,
  () => {
    clearSelection();
  },
  { deep: true },
);

watch(
  () => [props.loading, props.error],
  () => {
    clearSelection();
  },
);
const onCreate = () => emit('create');
const onView = (row: T) => {
  if (row && !props.loading && !props.error) emit('view', getRowId(row));
};
const onRowDblClick = (evt: Event, row: T) => {
  if (!props.hideView) {
    onView(row);
  }
};
const onEdit = () => {
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => getRowId(row));
    emit('edit', ids);
  }
};
const onDelete = () => {
  if (selected.value.length !== 1) {
    $q.notify({
      message: t('datatable.actions.deleteSingleOnly'),
      type: 'warning',
      position: 'top-right',
      timeout: 2500,
      actions: [{ icon: 'close', color: 'white', round: true, size: 'sm' }],
    });
    return;
  }

  emit('delete', [getRowId(selected.value[0] as T)]);
};
</script>

<style lang="scss">
.table-loading-row > td {
  height: auto;
  padding: 8px 16px;
}

.data-table-wrapper {
  background-color: var(--card-bg) !important;
  border-color: var(--border-color) !important;
  color: var(--text-color) !important;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;

  .custom-table {
    background-color: transparent !important;
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;

    .q-table__middle {
      flex: 1;
      min-height: 0;
      overflow: auto;
    }

    .dt-fixed-layout {
      table-layout: fixed;
      width: max-content;
      min-width: 100%;

      th,
      td {
        box-sizing: border-box;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      th > *,
      td > * {
        min-width: 0;
        max-width: 100%;
      }

      .q-chip {
        max-width: 100%;

        .q-chip__content {
          min-width: 0;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
      }
    }

    .q-table__top {
      position: sticky;
      top: 0;
      z-index: 5;
      background-color: var(--card-bg);
    }

    thead tr th {
      position: sticky;
      z-index: 2;
      background-color: var(--q-dark-page) !important;
      color: var(--text-color) !important;
      border-bottom: 1px solid var(--border-color) !important;
    }

    thead tr:first-child th {
      top: 0;
    }

    tbody tr td {
      height: var(--row-height);
      font-variant-numeric: tabular-nums;
      border-bottom: 1px solid var(--border-color) !important;
    }

    // Casillas del encabezado y de las filas: mismo ancho y centradas en la misma columna.
    .dt-selection-cell,
    tbody tr td.q-table--col-auto-width:first-child:has(> .q-checkbox) {
      width: 56px;
      min-width: 56px;
      padding-inline: 8px;
      text-align: center;
    }

    .q-table__bottom {
      border-top: 1px solid var(--border-color) !important;
    }
  }

  @media (max-width: 599px) {
    .dt-actions {
      margin-top: 8px;
      justify-content: flex-start !important;
    }
  }
}

.dt-columns-dialog {
  width: min(420px, 92vw);
  background: var(--card-bg) !important;
  border-color: var(--border-color) !important;
  color: var(--text-color) !important;

  .dt-columns-list {
    max-height: min(420px, 65vh);
    overflow: auto;
  }

  .q-item {
    color: var(--text-color);
  }
}
</style>

<style>
.data-table-wrapper {
  width: 100%;
  max-width: 100%;
  overflow: hidden;
}
.data-table-wrapper .q-table__top {
  padding: 24px;
}
.data-table-wrapper .q-table__top .text-h5 {
  font-size: 24px;
}
.data-table-wrapper .q-table__top .q-icon.q-mr-sm {
  background: var(--surface-selected);
  color: var(--text-accent);
  padding: 8px;
  border-radius: 10px;
}
.data-table-wrapper .q-table tbody tr.selected {
  background: var(--surface-selected);
}
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 9px;
  background: var(--surface-hover);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
}
.status-badge::before {
  content: '';
  width: 6px;
  height: 6px;
  background: currentColor;
  border-radius: 50%;
}
.status-badge[data-status='in_progress'],
.status-badge[data-status='open'],
.status-badge[data-status='active'] {
  background: var(--info-bg);
  color: var(--info-text);
}
.status-badge[data-status='review'],
.status-badge[data-status='pending'] {
  background: var(--warning-bg);
  color: var(--warning-text);
}
.status-badge[data-status='done'],
.status-badge[data-status='paid'],
.status-badge[data-status='approved'] {
  background: var(--success-bg);
  color: var(--success-text);
}
.status-badge[data-status='cancelled'],
.status-badge[data-status='blocked'],
.status-badge[data-status='declined'] {
  background: var(--error-bg);
  color: var(--error-text);
}
.cell-text {
  width: 100%;
  text-align: inherit;
  font: inherit;
  color: inherit;
  background: none;
  border: 0;
  padding: 0;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
}
.cell-full-text {
  max-width: min(480px, 85vw);
  overflow-wrap: anywhere;
  white-space: normal;
  background: var(--surface-raised);
  color: var(--text-primary);
}
.preference-error {
  background: var(--error-bg);
  color: var(--error-text);
  border-radius: 8px;
}
@media (max-width: 599px) {
  .data-table-wrapper .q-table__top {
    padding: 16px;
  }
  .dt-actions {
    flex-wrap: wrap;
    gap: 8px;
  }
}
</style>
