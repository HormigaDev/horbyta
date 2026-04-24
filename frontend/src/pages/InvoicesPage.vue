<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="invoices_table"
      :title="t('invoices.list.title')"
      icon="receipt_long"
      :columns="getInvoiceColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="invoiceFilterSchema"
      @request="onRequest"
      @view="(id) => $router.push(`/invoices/${id}`)"
      @create="() => $router.push('/invoices/new')"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
    >
      <template #body-cell-amount="props">
        <q-td :props="props" class="text-right">
          {{ CURRENCY_SYMBOLS[(props.row as InvoiceSummaryDto).currency_type] }} {{ props.value }}
        </q-td>
      </template>

      <template #body-cell-status="props">
        <q-td :props="props" class="text-center">
          <q-chip
            :color="invoiceStatusColor((props.row as InvoiceSummaryDto).status)"
            text-color="white"
            dense
            class="text-caption text-weight-medium"
          >
            {{ props.value }}
          </q-chip>
        </q-td>
      </template>

      <template #body-cell-due_date="props">
        <q-td :props="props" class="text-center">
          <span
            v-if="props.value && props.value !== '-'"
            :class="isInvoiceOverdue(props.row as InvoiceSummaryDto) ? 'text-negative' : ''"
            class="row items-center justify-center no-wrap q-gutter-x-xs"
          >
            <q-icon v-if="isInvoiceOverdue(props.row as InvoiceSummaryDto)" name="schedule" size="xs" />
            <span>{{ props.value }}</span>
          </span>
          <span v-else class="text-grey-5">—</span>
        </q-td>
      </template>

      <template #filter-value-select-by-term="slotProps">
        <SelectByTerm
          v-if="slotProps.columnKey === 'project_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('invoices.list.projectLabel')"
          :options="projectFilterOptions"
          :loading="isProjectFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onProjectFilterSearch"
        />
      </template>
    </DataTable>
  </q-page>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { getInvoiceColumns } from 'src/data/columns';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { InvoicesService } from 'src/services/invoices';
import { ProjectsService } from 'src/services/projects';
import type { FilterSchema } from 'src/types/query';
import type { ProjectSummaryDto } from 'src/types/project';
import type { InvoiceSummaryDto } from 'src/types/invoice';
import { CURRENCY_SYMBOLS } from 'src/data/constants';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();

const invoiceStatusColor = (status: string): string =>
  ({ draft: 'grey-5', open: 'primary', paid: 'positive', void: 'grey-7', uncollectible: 'deep-orange' }[status] ?? 'grey');

const isInvoiceOverdue = (row: InvoiceSummaryDto): boolean => {
  if (!row.due_date || row.status !== 'open') return false;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const due = new Date(row.due_date);
  due.setHours(0, 0, 0, 0);
  return due < today;
};

const {
  options: projectFilterOptions,
  isLoading: isProjectFilterLoading,
  load: loadProjectFilterByTerm,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const invoiceFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('invoices.list.filter.id') },
  record_number: { type: 'Number', label: t('invoices.list.filter.recordNumber') },
  project_id: { type: 'Text', label: t('invoices.list.filter.project'), useSelectByTerm: true },
  currency_type: { type: 'Text', label: t('invoices.list.filter.currency') },
  status: { type: 'Text', label: t('invoices.list.filter.status') },
  due_date: { type: 'Date', label: t('invoices.list.filter.dueDate') },
  issued_at: { type: 'Date', label: t('invoices.list.filter.issuedAt') },
  created_at: { type: 'Date', label: t('invoices.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('invoices.list.filter.updatedAt') },
};

const { items, total, isLoading, query, fetch } = useQuery(
  InvoicesService.search,
  invoiceFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({ query, fetch });

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    router.push(`/invoices/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('invoices.list.selectOneToEdit'), 'warning');
  }
};

const onProjectFilterSearch = (term: string) => {
  if (!term.trim()) return;
  loadProjectFilterByTerm(term);
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => InvoicesService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('invoices.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('invoices.list.deleteSuccess'),
  buildDeleteError: () => t('invoices.list.deleteError'),
  afterDelete: fetch,
});
</script>
