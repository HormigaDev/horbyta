<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="clients_table"
      :title="$t('clients.list.title')"
      icon="people"
      :columns="getClientColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :error="error"
      :filter-schema="CLIENT_FILTER_SCHEMA"
      @request="onRequest"
      @view="(id) => $router.push(`/clients/${id}`)"
      @create="() => $router.push('/clients/new')"
      @refresh="fetch"
      @edit="editSelected"
      @delete="confirmDestruction"
    />
  </q-page>
</template>

<script setup lang="ts">
import DataTable from 'src/components/DataTable.vue';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { getClientColumns } from 'src/data/columns';
import { ClientsService } from 'src/services/clients';
import { CLIENT_FILTER_SCHEMA } from 'src/types/filterSchemas';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useCrudDelete } from 'src/composables/useCrudDelete';

const router = useRouter();
const { notify, notifyError, confirmDelete } = useUI();
const { t } = useI18n();
const { items, total, isLoading, error, query, fetch } = useQuery(
  ClientsService.search,
  CLIENT_FILTER_SCHEMA,
  10,
  'created_at',
  false,
);
const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    router.push(`/clients/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('clients.list.selectOneToEdit'), 'warning');
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => ClientsService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('clients.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('clients.list.deleteSuccess'),
  buildDeleteError: () => t('clients.list.deleteError'),
  afterDelete: fetch,
});
</script>
