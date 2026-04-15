<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="milestones_management_table"
      :title="t('milestones.list.title')"
      :columns="getMilestoneColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="milestoneFilterSchema"
      @request="onRequest"
      @view="onView"
      @create="onCreate"
      @edit="editSelected"
      @delete="confirmDestruction"
      @refresh="fetch"
    >
      <template #filter-value-select-by-term="slotProps">
        <SelectByTerm
          v-if="slotProps.columnKey === 'project_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('milestones.list.projectLabel')"
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
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useCrudDelete } from 'src/composables/useCrudDelete';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { useUI } from 'src/composables/useUI';
import { getMilestoneColumns } from 'src/data/columns';
import { MilestonesService } from 'src/services/milestones';
import { ProjectsService } from 'src/services/projects';
import type { MilestoneSummaryDto } from 'src/types/milestone';
import type { ProjectSummaryDto } from 'src/types/project';
import type { FilterSchema } from 'src/types/query';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();

const milestoneFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('milestones.list.filter.id') },
  project_id: {
    type: 'Text',
    label: t('milestones.list.filter.project'),
    useSelectByTerm: true,
  },
  name: { type: 'Text', label: t('milestones.list.filter.name') },
  status: { type: 'Text', label: t('milestones.list.filter.status') },
  due_date: { type: 'Date', label: t('milestones.list.filter.dueDate') },
  created_at: { type: 'Date', label: t('milestones.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('milestones.list.filter.updatedAt') },
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

const { items, total, isLoading, query, fetch } = useQuery<MilestoneSummaryDto>(
  MilestonesService.search,
  milestoneFilterSchema,
  10,
  'created_at',
  false,
);

const { onRequest } = useCrudListHandlers({
  query,
  fetch,
});

const onProjectFilterSearch = (term: string) => {
  void loadProjectFilterByTerm(term);
};

const onView = (id: string) => {
  void router.push(`/milestones/${id}`);
};

const onCreate = () => {
  void router.push('/milestones/new');
};

const editSelected = (selectedIds: string[]) => {
  if (selectedIds.length === 1) {
    void router.push(`/milestones/${selectedIds[0]}/edit`);
    return;
  }

  if (selectedIds.length > 1) {
    notify(t('milestones.list.selectOneToEdit'), 'warning');
  }
};

const { deleteSelected: confirmDestruction } = useCrudDelete({
  deleteById: (id) => MilestonesService.delete(id),
  confirmDelete,
  notify,
  notifyError,
  buildDeleteTarget: (count) => t('milestones.list.deleteTarget', { count }),
  buildDeleteSuccess: () => t('milestones.list.deleteSuccess'),
  buildDeleteError: () => t('milestones.list.deleteError'),
  afterDelete: fetch,
});

onMounted(() => {
  void loadProjectFilterByTerm('');
});
</script>
