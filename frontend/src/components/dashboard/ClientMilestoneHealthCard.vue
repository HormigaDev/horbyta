<template>
  <DashboardChartCardWithSettings
    :title="t('clientDashboard.milestones.title')"
    :caption="t('clientDashboard.milestones.caption')"
    short
    auto-height
    :filters-title="t('dashboard.filters.clientMilestoneHealth.title')"
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
    <ClientMilestoneHealthList
      :rows="state.rows.value"
      :empty-label="t('clientDashboard.milestones.empty')"
      :total-label="t('clientDashboard.milestones.total')"
      :completed-label="t('clientDashboard.milestones.completed')"
      :upcoming-label="t('clientDashboard.milestones.upcoming')"
      :overdue-label="t('clientDashboard.milestones.overdue')"
      :next-due-date-label="t('clientDashboard.milestones.nextDueDate')"
      :empty-value="t('common.empty')"
    />

    <template #filters>
      <SelectByTerm
        v-model="state.draftFilters.project_id"
        :options="projectOptions"
        :loading="isProjectsLoading"
        :label="t('dashboard.filters.common.project')"
        @search="(term) => loadProjects(term)"
      />
      <q-select
        v-model="state.draftFilters.milestone_status"
        :options="milestoneStatusOptions"
        :label="t('dashboard.filters.clientMilestoneHealth.milestoneStatus')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
      <DashboardDateFilter
        v-model="state.draftFilters.due_date_from"
        :label="t('dashboard.filters.clientMilestoneHealth.dueDateFrom')"
      />
      <DashboardDateFilter
        v-model="state.draftFilters.due_date_to"
        :label="t('dashboard.filters.clientMilestoneHealth.dueDateTo')"
      />
      <q-toggle
        :model-value="state.draftFilters.overdue_only ?? false"
        :label="t('dashboard.filters.clientMilestoneHealth.overdueOnly')"
        @update:model-value="(value) => (state.draftFilters.overdue_only = value)"
      />
      <q-toggle
        :model-value="state.draftFilters.upcoming_only ?? false"
        :label="t('dashboard.filters.clientMilestoneHealth.upcomingOnly')"
        @update:model-value="(value) => (state.draftFilters.upcoming_only = value)"
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
import { ClientDashboardService } from 'src/services/clientDashboard';
import { ClientProjectsService } from 'src/services/clientProjects';
import type { ClientMilestoneHealthRow } from 'src/types/dashboard';
import type { ClientMilestoneHealthFiltersDto } from 'src/types/dashboardFilters';
import type { ClientProjectSummaryDto } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import ClientMilestoneHealthList from './ClientMilestoneHealthList.vue';
import DashboardDateFilter from './DashboardDateFilter.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  initialRows: ClientMilestoneHealthRow[];
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const milestoneStatusOptions = computed(() => [
  { label: t('milestones.status.open'), value: 'open' },
  { label: t('milestones.status.review'), value: 'review' },
  { label: t('milestones.status.approved'), value: 'approved' },
  { label: t('milestones.status.released'), value: 'released' },
  { label: t('milestones.status.cancelled'), value: 'cancelled' },
]);

const defaults = (): ClientMilestoneHealthFiltersDto => ({
  project_id: null,
  milestone_status: null,
  due_date_from: null,
  due_date_to: null,
  overdue_only: null,
  upcoming_only: null,
});

const state = useChartCardState<ClientMilestoneHealthFiltersDto, ClientMilestoneHealthRow>({
  defaults,
  fetchRows: (filters) => ClientDashboardService.getMilestoneHealth(filters),
  initialRows: () => props.initialRows,
});

const {
  options: projectOptions,
  isLoading: isProjectsLoading,
  load: loadProjects,
} = useSelectByTerm<ClientProjectSummaryDto>(ClientProjectsService.search, {
  termColumn: 'name',
  toOption: (project) => ({ label: project.name, value: project.id }),
});

const onApply = async () => {
  try {
    await state.applyDraft();
  } catch (error) {
    notifyError(t('clientDashboard.error'), error);
  }
};

const onReset = async () => {
  try {
    await state.resetDraft();
  } catch (error) {
    notifyError(t('clientDashboard.error'), error);
  }
};
</script>
