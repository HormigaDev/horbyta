<template>
  <q-page class="q-pa-md flex column">
    <DataTable
      class="col"
      table-id="tasks_table"
      :title="t('tasks.list.title')"
      :columns="getTaskColumns($t)"
      :rows="items"
      :totalItems="total"
      :loading="isLoading"
      :filter-schema="taskFilterSchema"
      @request="onRequest"
      @filters="onFilters"
      @view="onView"
      @create="handleCreate"
      @edit="handleEdit"
      @delete="confirmDestruction"
      @refresh="fetchTasks"
    >
      <template #filter-value-select-by-term="slotProps">
        <SelectByTerm
          v-if="slotProps.columnKey === 'project_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('tasks.list.projectLabel')"
          :options="projectFilterOptions"
          :loading="isProjectFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onProjectFilterSearch"
        />

        <SelectByTerm
          v-else-if="slotProps.columnKey === 'milestone_id'"
          :model-value="typeof slotProps.filter.value === 'string' ? slotProps.filter.value : null"
          :label="t('tasks.list.milestoneLabel')"
          :options="milestoneFilterOptions"
          :loading="isMilestoneFilterLoading"
          @update:model-value="
            (value) => slotProps.setValue(typeof value === 'string' ? value : '')
          "
          @search="onMilestoneFilterSearch"
        />
      </template>
    </DataTable>
  </q-page>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { useUI } from 'src/composables/useUI';
import { TasksService } from 'src/services/tasks';
import { ProjectsService } from 'src/services/projects';
import { MilestonesService } from 'src/services/milestones';
import type { Filter, FilterSchema, QueryPayload } from 'src/types';
import type { Milestone } from 'src/types/milestone';
import type { ProjectSummary } from 'src/types/project';
import type { TaskSummary } from 'src/types/task';
import { getTaskColumns } from 'src/data/columns';

const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete } = useUI();

const taskFilterSchema: FilterSchema = {
  id: { type: 'Text', label: t('tasks.list.filter.id') },
  project_id: { type: 'Text', label: t('tasks.list.filter.project'), useSelectByTerm: true },
  parent_id: { type: 'Text', label: t('tasks.list.filter.parentTask') },
  milestone_id: { type: 'Text', label: t('tasks.list.filter.milestone'), useSelectByTerm: true },
  title: { type: 'Text', label: t('tasks.list.filter.title') },
  status: { type: 'Text', label: t('tasks.list.filter.status') },
  priority: { type: 'Number', label: t('tasks.list.filter.priority') },
  complexity: { type: 'Number', label: t('tasks.list.filter.complexity') },
  block_parent: { type: 'Boolean', label: t('tasks.list.filter.blockParent') },
  due_date: { type: 'Date', label: t('tasks.list.filter.dueDate') },
  created_at: { type: 'Date', label: t('tasks.list.filter.createdAt') },
  updated_at: { type: 'Date', label: t('tasks.list.filter.updatedAt') },
};

const {
  options: projectFilterOptions,
  isLoading: isProjectFilterLoading,
  load: loadProjectFilterByTerm,
} = useSelectByTerm<ProjectSummary>(ProjectsService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (project) => ({
    label: project.name,
    value: project.id,
  }),
});

const {
  options: milestoneFilterOptions,
  isLoading: isMilestoneFilterLoading,
  load: loadMilestoneFilterByTerm,
} = useSelectByTerm<Milestone>(MilestonesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (milestone) => ({
    label: milestone.name,
    value: milestone.id,
  }),
});

const query = reactive<QueryPayload>({
  page: 1,
  limit: 10,
  order_by: 'created_at',
  order_asc: false,
  filters: [],
});

const items = ref<TaskSummary[]>([]);
const total = ref(0);
const isLoading = ref(false);
const hasLoadedInitialView = ref(false);

const loadView = async () => {
  isLoading.value = true;
  try {
    const view = await TasksService.view();
    items.value = view.items;
    total.value = view.total;
  } catch (err: unknown) {
    notifyError(t('tasks.list.loadInitialError'), err);
  } finally {
    isLoading.value = false;
  }
};

const fetchTasks = async () => {
  isLoading.value = true;
  try {
    const payload: QueryPayload = {
      page: query.page,
      limit: query.limit,
      order_by: query.order_by,
      order_asc: query.order_asc,
      filters: [...query.filters],
    };

    const response = await TasksService.search(payload);
    items.value = response.items;
    total.value = response.total;
  } catch (err: unknown) {
    notifyError(t('tasks.list.fetchError'), err);
  } finally {
    isLoading.value = false;
  }
};

const onRequest = (props: Record<string, unknown>) => {
  const pagination = props.pagination as {
    page: number;
    rowsPerPage: number;
    sortBy: string;
    descending: boolean;
  };

  if (pagination.rowsPerPage && pagination.rowsPerPage !== query.limit) {
    query.limit = pagination.rowsPerPage;
    query.page = 1;
  }

  if (pagination.page && pagination.page !== query.page) {
    query.page = pagination.page;
  }

  if (pagination.sortBy) {
    query.order_by = pagination.sortBy;
    query.order_asc = !pagination.descending;
  }

  if (!hasLoadedInitialView.value && query.page === 1 && query.filters.length === 0) {
    hasLoadedInitialView.value = true;
    void loadView();
    return;
  }

  void fetchTasks();
};

const onFilters = (filters: Filter[]) => {
  query.filters = filters;
  query.page = 1;
  hasLoadedInitialView.value = true;
  void fetchTasks();
};

const onProjectFilterSearch = (term: string) => {
  void loadProjectFilterByTerm(term);
};

const onMilestoneFilterSearch = (term: string) => {
  void loadMilestoneFilterByTerm(term);
};

const onView = (id: string) => {
  void router.push(`/tasks/${id}`);
};

const handleCreate = () => {
  void router.push('/tasks/new');
};

const handleEdit = (ids: string[]) => {
  if (!ids[0]) return;

  if (ids.length > 1) {
    notify(t('tasks.list.selectOneToEdit'), 'warning');
    return;
  }

  void router.push(`/tasks/${ids[0]}/edit`);
};

const confirmDestruction = async (selectedIds: string[]) => {
  if (selectedIds.length === 0) return;

  const confirmed = await confirmDelete(
    t('tasks.list.deleteTarget', { count: selectedIds.length }),
  );
  if (!confirmed) return;

  try {
    for (const id of selectedIds) {
      await TasksService.delete(id);
    }

    notify(t('tasks.list.deleteSuccess'), 'positive');
    void fetchTasks();
  } catch (err: unknown) {
    notifyError(t('tasks.list.deleteError'), err);
  }
};
</script>
