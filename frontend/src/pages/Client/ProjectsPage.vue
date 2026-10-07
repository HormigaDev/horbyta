<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="client_projects_table"
      :title="t('projects.list.title')"
      :columns="getClientProjectColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :error="error"
      :filter-schema="clientProjectFilterSchema"
      selection-mode="single"
      hide-create
      hide-edit
      hide-delete
      @request="onRequest"
      @view="(id) => $router.push(`/client/projects/${id}`)"
      @refresh="fetch"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import { getClientProjectColumns } from 'src/data/columns';
import { useQuery } from 'src/composables/useQuery';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { ClientProjectsService } from 'src/services/clientProjects';
import type { FilterSchema } from 'src/types';

const { t } = useI18n();

const clientProjectFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('projects.list.filter.id') },
  name: { type: 'Text', label: t('projects.list.filter.name') },
  status: { type: 'Text', label: t('projects.list.filter.status') },
  created_at: { type: 'Date', label: t('projects.list.filter.created') },
  updated_at: { type: 'Date', label: t('projects.list.filter.updated') },
};

const { items, total, isLoading, error, query, fetch } = useQuery(
  ClientProjectsService.search,
  clientProjectFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});
</script>
