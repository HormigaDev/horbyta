<template>
  <DashboardChartCardWithSettings
    :title="t('dashboard.bottlenecks.title')"
    :caption="t('dashboard.bottlenecks.caption')"
    short
    auto-height
    :filters-title="t('dashboard.filters.taskBottlenecks.title')"
    :open="state.dialogOpen.value"
    :loading="state.isLoading.value"
    :error="!!state.loadError.value"
    :filter-count="
      Object.values(state.appliedFilters).filter(
        (value) => value !== null && value !== undefined && value !== '',
      ).length
    "
    @retry="state.refetch().catch(() => {})"
    @open="state.openDialog"
    @close="state.dialogOpen.value = false"
    @apply="onApply"
    @reset="onReset"
  >
    <TaskBottlenecksList
      :rows="state.rows.value"
      :labels="bottleneckLabels"
      :empty-label="t('dashboard.bottlenecks.empty')"
    />

    <template #filters>
      <SelectByTerm
        v-model="state.draftFilters.client_id"
        :options="clientOptions"
        :loading="isClientsLoading"
        :label="t('dashboard.filters.common.client')"
        @search="(term) => loadClients(term)"
      />
      <SelectByTerm
        v-model="state.draftFilters.project_id"
        :options="projectOptions"
        :loading="isProjectsLoading"
        :label="t('dashboard.filters.common.project')"
        @search="(term) => loadProjects(term)"
      />
      <q-select
        v-model="state.draftFilters.status"
        :options="statusOptions"
        :label="t('dashboard.filters.taskBottlenecks.status')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
      <q-select
        v-model="state.draftFilters.priority"
        :options="priorityOptions"
        :label="t('dashboard.filters.taskBottlenecks.priority')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
      <q-toggle
        :model-value="state.draftFilters.overdue_only ?? false"
        :label="t('dashboard.filters.taskBottlenecks.overdueOnly')"
        @update:model-value="(value) => (state.draftFilters.overdue_only = value)"
      />
    </template>
  </DashboardChartCardWithSettings>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { useChartCardState } from 'src/composables/dashboard/useChartCardState';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ClientsService } from 'src/services/clients';
import { ProjectsService } from 'src/services/projects';
import { DashboardService } from 'src/services/dashboard';
import type { TaskBottleneckRow } from 'src/types/dashboard';
import type { TaskBottlenecksFiltersDto } from 'src/types/dashboardFilters';
import type { ClientSummaryDto } from 'src/types/clients';
import type { ProjectSummaryDto } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import TaskBottlenecksList, { type TaskBottlenecksLabels } from './TaskBottlenecksList.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  initialRows: TaskBottleneckRow[];
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const bottleneckLabels = computed<TaskBottlenecksLabels>(() => ({
  total: t('dashboard.bottlenecks.column.total'),
  todo: t('dashboard.bottlenecks.column.todo'),
  inProgress: t('dashboard.bottlenecks.column.inProgress'),
  review: t('dashboard.bottlenecks.column.review'),
  overdue: t('dashboard.bottlenecks.column.overdue'),
}));

const statusOptions = computed(() => [
  { label: t('dashboard.bottlenecks.column.todo'), value: 'todo' },
  { label: t('dashboard.bottlenecks.column.inProgress'), value: 'in_progress' },
  { label: t('dashboard.bottlenecks.column.review'), value: 'review' },
  { label: t('dashboard.filters.taskBottlenecks.statusDone'), value: 'done' },
  { label: t('dashboard.filters.taskBottlenecks.statusCancelled'), value: 'cancelled' },
]);

const priorityOptions = computed(() => [
  { label: t('dashboard.filters.taskBottlenecks.priorityLow'), value: 1 },
  { label: t('dashboard.filters.taskBottlenecks.priorityMedium'), value: 2 },
  { label: t('dashboard.filters.taskBottlenecks.priorityHigh'), value: 3 },
  { label: t('dashboard.filters.taskBottlenecks.priorityCritical'), value: 4 },
]);

const defaults = (): TaskBottlenecksFiltersDto => ({
  client_id: null,
  project_id: null,
  status: null,
  priority: null,
  overdue_only: null,
});

const state = useChartCardState<TaskBottlenecksFiltersDto, TaskBottleneckRow>({
  defaults,
  fetchRows: (filters) => DashboardService.getTaskBottlenecks(filters),
  initialRows: () => props.initialRows,
});

const {
  options: clientOptions,
  isLoading: isClientsLoading,
  load: loadClients,
} = useSelectByTerm<ClientSummaryDto>(ClientsService.search, {
  termColumn: 'name',
  toOption: (client) => ({ label: client.name, value: client.id }),
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjects,
} = useSelectByTerm<ProjectSummaryDto>(ProjectsService.search, {
  termColumn: 'name',
  toOption: (project) => ({ label: project.name, value: project.id }),
});

const onApply = async () => {
  try {
    await state.applyDraft();
  } catch (error) {
    notifyError(t('dashboard.error'), error);
  }
};

const onReset = async () => {
  try {
    await state.resetDraft();
  } catch (error) {
    notifyError(t('dashboard.error'), error);
  }
};
</script>
