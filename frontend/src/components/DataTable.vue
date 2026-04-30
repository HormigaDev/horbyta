<template>
  <q-card class="data-table-wrapper" flat bordered>
    <q-table
      separator="cell"
      :title="title"
      :rows="rows"
      :columns="columns"
      :row-key="rowKey"
      v-model:pagination="pagination"
      v-model:selected="selected"
      :selection="selectionMode"
      :loading="loading"
      :rows-per-page-options="[5, 10, 20, 50, 100]"
      table-class="dt-fixed-layout"
      :visible-columns="visibleColumns"
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
              :style="filters.length > 0 ? 'color: var(--q-primary) !important' : ''"
            >
              <q-tooltip>{{ t('datatable.tooltip.filters') }}</q-tooltip>
              <q-badge v-if="filters.length > 0" color="red" floating>{{ filters.length }}</q-badge>
            </q-btn>

            <q-btn
              flat
              round
              dense
              icon="view_column"
              style="color: var(--text-muted)"
              @click="openColumnDialog"
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
                :label="$q.screen.gt.xs ? t('datatable.actions.delete') : ''"
                @click="onDelete"
              />
            </div>
            <div v-else>
              <q-btn
                v-if="!hideCreate"
                unelevated
                color="primary"
                icon="add"
                :label="$q.screen.gt.xs ? t('datatable.actions.new') : ''"
                @click="onCreate"
              />
            </div>
          </div>
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

        <slot name="filters"></slot>
      </template>

      <template #header="props">
        <q-tr :props="props">
          <q-th auto-width class="text-center" v-if="selectionMode !== 'none'">
            <q-checkbox v-model="props.selected" />
          </q-th>
          <q-th
            v-for="col in props.cols"
            :key="col.name"
            :props="props"
            class="text-weight-bold text-center"
          >
            {{ translateLabel(col.label) }}
          </q-th>
        </q-tr>
      </template>

      <template #no-data>
        <div class="full-width row flex-center text-primary q-pa-xl text-center">
          <q-icon size="4em" name="inbox" style="color: var(--text-muted)" />
          <div class="text-h6 q-mt-sm" style="color: var(--text-muted)">
            {{ t('datatable.noDataTitle') }}
          </div>
        </div>
      </template>

      <template v-for="(_, slotName) in $slots" #[slotName]="slotProps" :key="slotName">
        <slot :name="slotName" v-bind="slotProps || {}" />
      </template>
    </q-table>

    <q-dialog v-model="showColumnDialog">
      <q-card flat bordered class="dt-columns-dialog">
        <q-card-section class="row items-center justify-between q-pb-sm">
          <div class="text-h6">{{ t('datatable.columnsDialogTitle') }}</div>
          <q-btn flat round dense icon="close" v-close-popup>
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
                  :val="column.value"
                  :disable="
                    draftVisibleColumns.length === 1 &&
                    draftVisibleColumns.includes(column.value)
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
import { ref, onMounted, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useDataTableStore } from '../stores/datatable';
import type { ColumnType, Filter, FilterOperator, FilterSchema } from '../types/query';
import type { QTableProps } from 'quasar';

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
  rowKey?: string;
  selectionMode?: SelectionMode;
  hideCreate?: boolean;
  hideView?: boolean;
  hideEdit?: boolean;
  hideDelete?: boolean;
  hideRefresh?: boolean;
  filterSchema?: FilterSchema;
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
});

const { t } = useI18n();

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
const selected = ref<T[]>([]);
const visibleColumns = ref(props.columns.map((c) => c.name));
const draftVisibleColumns = ref<string[]>([]);
const showColumnDialog = ref(false);
const isSavingColumnPreferences = ref(false);

const showFilters = ref(false);
const filters = ref<Partial<Filter>[]>([]);

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
    visibleColumns.value = nextVisibleColumns;
    await store.saveVisibleColumns(props.tableId, nextVisibleColumns);
    showColumnDialog.value = false;
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
  return filters.value.filter((f) => f.column && f.operator && f.value !== '') as Filter[];
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
  const savedState = await store.getTableState(props.tableId);

  if (savedState.pagination) {
    pagination.value = {
      ...defaultPagination,
      ...savedState.pagination,
      sortBy: savedState.pagination.sortBy || defaultPagination.sortBy,
      rowsNumber: props.totalItems,
    };
  }
  if (savedState.visibleColumns && savedState.visibleColumns.length > 0) {
    visibleColumns.value = [...savedState.visibleColumns];
  }
  if (savedState.filters) {
    filters.value = [...savedState.filters] as Partial<Filter>[];
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
  const requestFilters = requestProps.filters ?? getValidFilters();

  const sortChanged =
    pagination.value.sortBy !== sortBy || pagination.value.descending !== descending;

  pagination.value.page = page;
  pagination.value.rowsPerPage = rowsPerPage;
  pagination.value.sortBy = sortBy;
  pagination.value.descending = descending;

  void store.saveTableState(
    props.tableId,
    {
      pagination: { page, rowsPerPage, sortBy, descending },
      visibleColumns: visibleColumns.value,
      filters: requestFilters,
    },
  );

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
  (newRows) => {
    if (selected.value.length > 0 && Array.isArray(newRows)) {
      const rowIds = new Set(newRows.map((row) => getRowValue(row, props.rowKey)));
      selected.value = selected.value.filter((row) => rowIds.has(getRowValue(row, props.rowKey)));
    }
  },
  { deep: true },
);

const onCreate = () => emit('create');
const onView = (row: T) => {
  if (row) emit('view', getRowId(row));
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
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => getRowId(row));
    emit('delete', ids);
  }
};
</script>

<style lang="scss">
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
      width: 100%;
    }

    thead tr th {
      position: sticky;
      z-index: 2;
      background-color: var(--q-dark-page) !important;
      color: var(--text-color) !important;
      border-bottom: 1px solid var(--border-color) !important;
      text-align: center !important;
    }

    thead tr:first-child th {
      top: 0;
    }

    tbody tr td {
      border-bottom: 1px solid var(--border-color) !important;
    }

    tbody tr td.q-table--col-auto-width {
      text-align: center !important;
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
