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
      @view="(id) => $router.push(`/projects/${id}`)"
      @create="() => $router.push('/projects/new')"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
    >
      <template #filter-value-select-by-term="slotProps">
        <SelectByTerm
          v-if="slotProps.columnKey === 'category_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('projects.list.categoryLabel')"
          :options="categoryFilterOptions"
          :loading="isCategoryFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onCategoryFilterSearch"
        />

        <SelectByTerm
          v-else-if="slotProps.columnKey === 'client_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('projects.list.clientLabel')"
          :options="clientFilterOptions"
          :loading="isClientFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onClientFilterSearch"
        />
      </template>
    </DataTable>
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
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { CategoriesService } from 'src/services/categories';
import { ClientsService } from 'src/services/clients';
import type { FilterSchema } from 'src/types/query';
import type { Category } from 'src/types/category';
import type { Client } from 'src/types/clients';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { onMounted } from 'vue';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();

const {
  options: categoryFilterOptions,
  isLoading: isCategoryFilterLoading,
  load: loadCategoryFilterByTerm,
} = useSelectByTerm<Category>(CategoriesService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (category) => ({
    label: category.name,
    value: category.id,
  }),
});

const {
  options: clientFilterOptions,
  isLoading: isClientFilterLoading,
  load: loadClientFilterByTerm,
} = useSelectByTerm<Client>(ClientsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (client) => ({
    label: client.name,
    value: client.id,
  }),
});

const projectFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('projects.list.filter.id') },
  name: { type: 'Text', label: t('projects.list.filter.name') },
  category_id: { type: 'Text', label: t('projects.list.filter.category'), useSelectByTerm: true },
  client_id: { type: 'Text', label: t('projects.list.filter.client'), useSelectByTerm: true },
  created_at: { type: 'Date', label: t('projects.list.filter.created') },
  updated_at: { type: 'Date', label: t('projects.list.filter.updated') },
};

const { items, total, isLoading, query, fetch } = useQuery(
  ProjectsService.search,
  projectFilterSchema,
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
    router.push(`/projects/${selectedIds[0]}/edit`);
  } else if (selectedIds.length > 1) {
    notify(t('projects.list.selectOneToEdit'), 'warning');
  }
};

const onCategoryFilterSearch = (term: string) => {
  if (!term.trim()) return;
  loadCategoryFilterByTerm(term);
};
const onClientFilterSearch = (term: string) => {
  if (!term.trim()) return;
  loadClientFilterByTerm(term);
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

onMounted(() => loadCategoryFilterByTerm('').then(() => loadClientFilterByTerm('')));
</script>
