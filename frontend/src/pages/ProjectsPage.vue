<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="projects_table"
      :title="t('projects.list.title')"
      :columns="getProjectColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="projectFilterSchema"
      @request="onRequest"
      @filters="onFilters"
      @view="(id) => $router.push(`/projects/${id}`)"
      @create="() => $router.push('/projects/new')"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
    />
  </q-page>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import { getProjectColumns } from 'src/data/columns';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { ProjectsService } from 'src/services/projects';
import type { FilterSchema } from 'src/types/query';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();

const projectFilterSchema: FilterSchema = {
  id: 'Text',
  name: 'Text',
  description: 'Text',
  category_id: 'Text',
  created_at: 'Date',
  updated_at: 'Date',
};

const { items, total, isLoading, query, fetch } = useQuery(
  ProjectsService.search,
  projectFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest, onFilters } = useCrudListHandlers({
  query,
  fetch,
});

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    router.push(`/projects/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('projects.list.selectOneToEdit'), 'warning');
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => ProjectsService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('projects.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('projects.list.deleteSuccess'),
  buildDeleteError: () => t('projects.list.deleteError'),
  afterDelete: fetch,
});
</script>
