<template>
  <q-card class="data-table-wrapper" flat bordered>
    <q-table
      :title="title"
      :rows="rows"
      :columns="columns"
      :row-key="rowKey"
      v-model:pagination="pagination"
      v-model:selected="selected"
      :selection="selectionMode"
      :loading="loading"
      :rows-per-page-options="[5, 10, 20, 50, 100]"
      :visible-columns="visibleColumns"
      rows-per-page-label="Registros por página"
      no-results-label="No hay datos para mostrar"
      loading-label="Cargando registros..."
      :selected-rows-label="
        (numberOfRows: number) => {
          if (numberOfRows <= 0) return '';

          return numberOfRows > 1
            ? `${numberOfRows} Registros seleccionados`
            : `1 Registro seleccionado`;
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
          <div class="col-xs-12 col-sm text-subtitle1 text-weight-bold" v-if="title">
            {{ title }}
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
              <q-tooltip>Actualizar Datos</q-tooltip>
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
              <q-tooltip>Filtros</q-tooltip>
              <q-badge v-if="filters.length > 0" color="red" floating>{{ filters.length }}</q-badge>
            </q-btn>

            <q-select
              v-model="visibleColumns"
              multiple
              outlined
              dense
              options-dense
              :display-value="$q.screen.gt.xs ? 'Columnas' : ''"
              emit-value
              map-options
              :options="columns"
              option-value="name"
              style="min-width: 130px; max-width: 200px"
              class="dt-column-select"
            >
              <template v-slot:option="{ itemProps, opt, selected, toggleOption }">
                <q-item
                  v-bind="itemProps"
                  @click="toggleOption(opt)"
                  :style="
                    selected ? 'color: var(--q-primary); background-color: var(--bg-hover)' : ''
                  "
                >
                  <q-item-section>
                    <q-item-label>{{ opt.label }}</q-item-label>
                  </q-item-section>
                  <q-item-section side v-if="selected">
                    <q-icon name="check" color="primary" />
                  </q-item-section>
                </q-item>
              </template>
            </q-select>

            <q-separator vertical class="q-mx-sm" v-if="selected.length > 0" />

            <div class="row q-gutter-x-sm" v-if="selected.length > 0">
              <q-btn
                v-if="!hideView && selected.length === 1"
                outline
                color="secondary"
                icon="visibility"
                label="Ver"
                @click="onView(selected[0] as T)"
              />
              <q-btn
                v-if="!hideEdit && selected.length === 1"
                outline
                color="primary"
                icon="edit"
                label="Editar"
                @click="onEdit"
              />
              <q-btn
                v-if="!hideDelete"
                outline
                color="negative"
                icon="delete"
                :label="$q.screen.gt.xs ? 'Eliminar' : ''"
                @click="onDelete"
              />
            </div>
            <div v-else>
              <q-btn
                v-if="!hideCreate"
                unelevated
                color="primary"
                icon="add"
                :label="$q.screen.gt.xs ? 'Nuevo' : ''"
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
                  label="Columna"
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
                  label="Operador"
                  dense
                  outlined
                  emit-value
                  map-options
                  :disable="!filter.column"
                />
              </div>
              <div class="col-12 col-sm-5">
                <q-input
                  v-model="filter.value"
                  label="Valor"
                  dense
                  outlined
                  :type="getFilterInputType(filter.column)"
                  :disable="!filter.operator"
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
                  <q-tooltip>Eliminar filtro</q-tooltip>
                </q-btn>
              </div>
            </div>

            <div class="row justify-between q-mt-sm">
              <q-btn
                outline
                color="primary"
                icon="add"
                label="Agregar Filtro"
                size="sm"
                @click="addFilter"
                :disable="filters.length >= 5"
              />
              <div class="row q-gutter-x-sm">
                <q-btn
                  flat
                  color="negative"
                  label="Limpiar Todos"
                  size="sm"
                  @click="clearFilters"
                  v-if="filters.length > 0"
                />
                <q-btn
                  unelevated
                  color="primary"
                  label="Aplicar Filtros"
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
          <q-th auto-width v-if="selectionMode !== 'none'">
            <q-checkbox v-model="props.selected" />
          </q-th>
          <q-th v-for="col in props.cols" :key="col.name" :props="props" class="text-weight-bold">
            {{ col.label }}
          </q-th>
        </q-tr>
      </template>

      <template #no-data>
        <div class="full-width row flex-center text-primary q-pa-xl text-center">
          <q-icon size="4em" name="inbox" style="color: var(--text-muted)" />
          <div class="text-h6 q-mt-sm" style="color: var(--text-muted)">
            No hay registros para mostrar
          </div>
        </div>
      </template>

      <template v-for="(_, slotName) in $slots" #[slotName]="slotProps" :key="slotName">
        <slot :name="slotName" v-bind="slotProps || {}" />
      </template>
    </q-table>
  </q-card>
</template>

<script setup lang="ts" generic="T extends Record<string, any>">
import { ref, onMounted, watch, computed } from 'vue';
import { useDataTableStore } from '../stores/datatable';
import type { Filter, FilterOperator, FilterSchema } from '../types/query';
import type { QTableProps } from 'quasar';

type SelectionMode = 'none' | 'single' | 'multiple';

interface Props {
  tableId: string;
  title?: string;
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

const emit = defineEmits<{
  (
    e: 'request',
    requestProps: {
      pagination: { page: number; rowsPerPage: number; sortBy: string | null; descending: boolean };
    },
  ): void;
  (e: 'create'): void;
  (e: 'view', id: string): void;
  (e: 'edit', ids: string[]): void;
  (e: 'delete', ids: string[]): void;
  (e: 'refresh'): void;
  (e: 'selection-change', selected: T[]): void;
  (e: 'sort', val: { sortBy: string | null; descending: boolean }): void;
  (e: 'filters', filters: Filter[]): void;
}>();

const store = useDataTableStore();
const selected = ref<T[]>([]);
const visibleColumns = ref(props.columns.map((c) => c.name));

const showFilters = ref(false);
const filters = ref<Partial<Filter>[]>([]);

const filterColumnOptions = computed(() => {
  return Object.keys(props.filterSchema).map((key) => {
    const col = props.columns.find((c) => c.name === key || c.field === key);
    return {
      label: col ? col.label : key,
      value: key,
    };
  });
});

const getFilterOperators = (columnKey?: string) => {
  if (!columnKey) return [];
  const type = props.filterSchema[columnKey];

  const options = [
    { label: 'Igual (=)', value: '=' },
    { label: 'Distinto (!=)', value: '!=' },
  ];

  if (type === 'Number' || type === 'Date') {
    options.push(
      { label: 'Mayor (>)', value: '>' },
      { label: 'Menor (<)', value: '<' },
      { label: 'Mayor o igual (>=)', value: '>=' },
      { label: 'Menor o igual (<=)', value: '<=' },
    );
  }

  if (type === 'Text') {
    options.push(
      { label: 'Contiene (><)', value: '><' },
      { label: 'No contiene (<>)', value: '<>' },
    );
  }

  if (type === 'Date') {
    options.push({ label: 'Entre ()', value: '()' });
  }

  return options;
};

const getFilterInputType = (columnKey?: string) => {
  if (!columnKey) return 'text';
  const type = props.filterSchema[columnKey];
  if (type === 'Number') return 'number';
  if (type === 'Date') return 'date';
  return 'text';
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

const applyFilters = () => {
  const validFilters = filters.value.filter(
    (f) => f.column && f.operator && f.value !== '',
  ) as Filter[];
  store.saveTableState(props.tableId, { filters: validFilters });
  emit('filters', validFilters);
  onRequest({ pagination: pagination.value });
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

onMounted(() => {
  const savedState = store.getTableState(props.tableId);

  if (savedState.pagination) {
    pagination.value = {
      ...defaultPagination,
      ...savedState.pagination,
      rowsNumber: props.totalItems,
    };
  }
  if (savedState.visibleColumns && savedState.visibleColumns.length > 0) {
    visibleColumns.value = [...savedState.visibleColumns];
  }
  if (savedState.filters) {
    filters.value = [...savedState.filters] as Partial<Filter>[];
    emit('filters', savedState.filters);
  }

  onRequest({ pagination: pagination.value });
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
}) => {
  const { page, rowsPerPage, sortBy, descending } = requestProps.pagination;

  const sortChanged =
    pagination.value.sortBy !== sortBy || pagination.value.descending !== descending;

  pagination.value.page = page;
  pagination.value.rowsPerPage = rowsPerPage;
  pagination.value.sortBy = sortBy;
  pagination.value.descending = descending;

  store.saveTableState(props.tableId, {
    pagination: { page, rowsPerPage, sortBy, descending },
    visibleColumns: visibleColumns.value,
  });

  emit('request', requestProps);

  if (sortChanged) {
    emit('sort', { sortBy, descending });
  }
};

watch(visibleColumns, (cols) => {
  store.saveTableState(props.tableId, { visibleColumns: cols });
});

watch(
  () => props.rows,
  (newRows) => {
    if (selected.value.length > 0 && Array.isArray(newRows)) {
      const rowIds = new Set(newRows.map((r) => r[props.rowKey]));
      selected.value = selected.value.filter((s) => rowIds.has(s[props.rowKey]));
    }
  },
  { deep: true },
);

const onCreate = () => emit('create');
const onView = (row: T) => {
  if (row) emit('view', String(row[props.rowKey]));
};
const onRowDblClick = (evt: Event, row: T) => {
  if (!props.hideView) {
    onView(row);
  }
};
const onEdit = () => {
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => String(row[props.rowKey]));
    emit('edit', ids);
  }
};
const onDelete = () => {
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => String(row[props.rowKey]));
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
      overflow-y: auto;
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
      border-bottom: 1px solid var(--border-color) !important;
    }

    .q-table__bottom {
      border-top: 1px solid var(--border-color) !important;
    }

    .dt-column-select {
      .q-field__control {
        border-color: var(--border-color);
        background: var(--q-dark-page);
        color: var(--text-color);
        * {
          color: inherit !important;
        }
      }
    }
  }

  @media (max-width: 599px) {
    .dt-actions {
      margin-top: 8px;
      justify-content: flex-start !important;
    }
  }
}
</style>
