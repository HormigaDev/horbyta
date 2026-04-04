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
      @request="onRequest"
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

const { items, total, isLoading, query, fetch, setPage, setLimit } = useQuery(
  ProjectsService.search,
  projectFilterSchema,
  10,
  'created_at',
  false,
);

// Table request handler mapping from Quasar
const onRequest = (props: Record<string, unknown>) => {
  const pagination = props.pagination as {
    page: number;
    rowsPerPage: number;
    sortBy: string;
    descending: boolean;
  };

  if (pagination.rowsPerPage && pagination.rowsPerPage !== query.limit) {
    setLimit(pagination.rowsPerPage);
  }

  if (pagination.page && pagination.page !== query.page) {
    setPage(pagination.page);
  }

  if (pagination.sortBy) {
    if (query.order_by !== pagination.sortBy) {
      query.order_by = pagination.sortBy;
      query.order_asc = !pagination.descending;
      fetch();
    } else if (query.order_asc === pagination.descending) {
      query.order_asc = !pagination.descending;
      fetch();
    }
  } else {
    fetch();
  }
};

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    router.push(`/projects/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('projects.list.selectOneToEdit'), 'warning');
  }
};

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(
    t('projects.list.deleteTarget', { count: selectedIds.length }),
  );
  if (confirmed) {
    try {
      for (const id of selectedIds) {
        await ProjectsService.delete(id);
      }
      notify(t('projects.list.deleteSuccess'), 'positive');
      void fetch();
    } catch (err: unknown) {
      notifyError(t('projects.list.deleteError'), err);
    }
  }
};
</script>
