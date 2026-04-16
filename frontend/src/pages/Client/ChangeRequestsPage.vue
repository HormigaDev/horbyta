<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="client_change_requests_table"
      :title="t('client.changeRequests.list.title')"
      :columns="getClientChangeRequestColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :filter-schema="clientChangeRequestFilterSchema"
      selection-mode="single"
      hide-edit
      hide-delete
      @request="onRequest"
      @view="(id) => $router.push(`/client/change-requests/${id}`)"
      @create="() => $router.push('/client/change-requests/new')"
      @refresh="fetch"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import { getClientChangeRequestColumns } from 'src/data/columns';
import { useQuery } from 'src/composables/useQuery';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { ClientChangeRequestsService } from 'src/services/clientChangeRequests';
import type { FilterSchema } from 'src/types';

const { t } = useI18n();

const clientChangeRequestFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('changeRequests.list.filter.id') },
  record_number: { type: 'Number', label: t('changeRequests.list.filter.recordNumber') },
  project_id: { type: 'Text', label: t('changeRequests.list.filter.project') },
  title: { type: 'Text', label: t('changeRequests.list.filter.title') },
  status: { type: 'Text', label: t('changeRequests.list.filter.status') },
  created_at: { type: 'Date', label: t('changeRequests.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('changeRequests.list.filter.updatedAt') },
};

const { items, total, isLoading, query, fetch } = useQuery(
  ClientChangeRequestsService.search,
  clientChangeRequestFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});
</script>
