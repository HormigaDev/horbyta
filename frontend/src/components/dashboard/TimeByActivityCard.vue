<template>
  <DashboardChartCardWithSettings
    :title="t('dashboard.charts.timeByActivity.title')"
    :caption="t('dashboard.charts.timeByActivity.caption')"
    :filters-title="t('dashboard.filters.timeByActivity.title')"
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
    <TimeByActivityChart :rows="state.rows.value" :palette="palette" :empty-label="emptyLabel" />

    <template #filters>
      <DashboardDateFilter
        v-model="state.draftFilters.date_from"
        :label="t('dashboard.filters.common.dateFrom')"
      />
      <DashboardDateFilter
        v-model="state.draftFilters.date_to"
        :label="t('dashboard.filters.common.dateTo')"
      />
      <SelectByTerm
        v-model="state.draftFilters.activity_id"
        :options="activityOptions"
        :loading="isActivitiesLoading"
        :label="t('dashboard.filters.timeByActivity.activity')"
        @search="(term) => loadActivities(term)"
      />
      <q-select
        v-model="state.draftFilters.billable"
        :options="billableOptions"
        :label="t('dashboard.filters.timeByActivity.billable')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
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
    </template>
  </DashboardChartCardWithSettings>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { useChartCardState } from 'src/composables/dashboard/useChartCardState';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ActivitiesService } from 'src/services/activities';
import { ClientsService } from 'src/services/clients';
import { ProjectsService } from 'src/services/projects';
import { DashboardService } from 'src/services/dashboard';
import type { ChartPalette } from 'src/composables/dashboard/useChartPalette';
import type { TimeByActivityRow } from 'src/types/dashboard';
import type { TimeByActivityFiltersDto } from 'src/types/dashboardFilters';
import type { ActivitySummaryDto } from 'src/types/activity';
import type { ClientSummaryDto } from 'src/types/clients';
import type { ProjectSummaryDto } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import DashboardDateFilter from './DashboardDateFilter.vue';
import TimeByActivityChart from './TimeByActivityChart.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  palette: ChartPalette;
  emptyLabel: string;
  initialRows: TimeByActivityRow[];
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const billableOptions = computed(() => [
  { label: t('dashboard.filters.timeByActivity.billableOnly'), value: true },
  { label: t('dashboard.filters.timeByActivity.nonBillableOnly'), value: false },
]);

const defaults = (): TimeByActivityFiltersDto => ({
  date_from: null,
  date_to: null,
  activity_id: null,
  billable: null,
  client_id: null,
  project_id: null,
});

const state = useChartCardState<TimeByActivityFiltersDto, TimeByActivityRow>({
  defaults,
  fetchRows: (filters) => DashboardService.getTimeByActivity(filters),
  initialRows: () => props.initialRows,
});

const {
  options: activityOptions,
  isLoading: isActivitiesLoading,
  load: loadActivities,
} = useSelectByTerm<ActivitySummaryDto>(ActivitiesService.search, {
  termColumn: 'name',
  toOption: (activity) => ({ label: activity.name, value: activity.id }),
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
