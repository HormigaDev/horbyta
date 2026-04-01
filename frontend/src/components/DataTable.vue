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
          return numberOfRows > 1
            ? `${numberOfRows} Registros seleccionados`
            : `1 Registro seleccionado`;
        }
      "
      @request="onRequest"
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
              color="grey-7"
              @click="emit('refresh')"
            >
              <q-tooltip>Actualizar Datos</q-tooltip>
            </q-btn>

            <q-btn
              flat
              round
              dense
              icon="filter_alt"
              color="grey-7"
              @click="showFilters = !showFilters"
              :class="{ 'text-primary': filters.length > 0 }"
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
                  :class="selected ? 'text-primary bg-grey-3' : ''"
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
            class="q-mt-sm q-pa-sm bg-grey-2 rounded-borders full-width"
            style="
              border: 1px solid var(--border-color);
              background-color: var(--q-dark-page) !important;
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
          <q-icon size="4em" name="inbox" color="grey-5" />
          <div class="text-h6 text-grey-7 q-mt-sm">No hay registros para mostrar</div>
        </div>
      </template>

      <template v-for="(_, slotName) in $slots" #[slotName]="slotProps" :key="slotName">
        <slot :name="slotName" v-bind="slotProps || {}" />
      </template>
    </q-table>
  </q-card>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useDataTableStore } from '../stores/datatable';
import type { Filter, FilterOperator, FilterSchema } from '../types/query';
import type { QTableProps } from 'quasar';

type SelectionMode = 'none' | 'single' | 'multiple';

const props = defineProps({
  tableId: {
    type: String,
    required: true,
  },
  title: {
    type: String,
    default: '',
  },
  rows: {
    type: Array as () => Record<string, unknown>[],
    default: () => [],
  },
  columns: {
    type: Array as () => NonNullable<QTableProps['columns']>,
    required: true,
  },
  totalItems: {
    type: Number,
    default: 0,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  rowKey: {
    type: String,
    default: 'id',
  },
  selectionMode: {
    type: String as () => SelectionMode,
    default: 'multiple',
  },
  hideCreate: {
    type: Boolean,
    default: false,
  },
  hideEdit: {
    type: Boolean,
    default: false,
  },
  hideDelete: {
    type: Boolean,
    default: false,
  },
  hideRefresh: {
    type: Boolean,
    default: false,
  },
  filterSchema: {
    type: Object as () => FilterSchema,
    default: () => ({}),
  },
});

const emit = defineEmits([
  'request',
  'create',
  'edit',
  'delete',
  'refresh',
  'selection-change',
  'sort',
  'filters',
]);

const store = useDataTableStore();
const selected = ref<Record<string, unknown>[]>([]);
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

  // Desencadenar la primera carga
  onRequest({ pagination: pagination.value });
});

watch(
  () => props.totalItems,
  (newTotal) => {
    pagination.value.rowsNumber = newTotal;
  },
);

watch(selected, (newSel) => {
  emit('selection-change', newSel);
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

const onCreate = () => emit('create');
const onEdit = () => {
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => row[props.rowKey]);
    emit('edit', ids);
  }
};
const onDelete = () => {
  if (selected.value.length > 0) {
    const ids = selected.value.map((row) => row[props.rowKey]);
    emit('delete', ids);
  }
};
</script>

<style lang="scss">
.data-table-wrapper {
  background-color: var(--card-bg) !important;
  border-color: var(--border-color) !important;
  color: var(--text-color) !important;

  .custom-table {
    background-color: transparent !important;

    thead tr th {
      background-color: var(--q-dark-page) !important; // Cabeceras de tabla sutiles
      color: var(--text-color) !important;
      border-bottom: 1px solid var(--border-color) !important;
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
