<template>
  <DashboardChartCardWithSettings
    :title="t('dashboard.charts.monthlyRevenue.title')"
    :caption="t('dashboard.charts.monthlyRevenue.caption')"
    :filters-title="t('dashboard.filters.monthlyRevenue.title')"
    :open="state.dialogOpen.value"
    @open="state.openDialog"
    @close="state.dialogOpen.value = false"
    @apply="onApply"
    @reset="onReset"
  >
    <MonthlyRevenueChart :rows="state.rows.value" :palette="palette" :empty-label="emptyLabel" />

    <template #filters>
      <DashboardDateFilter
        v-model="state.draftFilters.date_from"
        :label="t('dashboard.filters.common.dateFrom')"
      />
      <DashboardDateFilter
        v-model="state.draftFilters.date_to"
        :label="t('dashboard.filters.common.dateTo')"
      />
      <q-select
        v-model="state.draftFilters.currency_type"
        :options="currencyOptions"
        :label="t('dashboard.filters.common.currencyType')"
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
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { useChartCardState } from 'src/composables/dashboard/useChartCardState';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { ClientsService } from 'src/services/clients';
import { ProjectsService } from 'src/services/projects';
import { DashboardService } from 'src/services/dashboard';
import { CURRENCY_TYPE_OPTIONS } from 'src/data/constants';
import type { ChartPalette } from 'src/composables/dashboard/useChartPalette';
import type { MonthlyRevenueRow } from 'src/types/dashboard';
import type { MonthlyRevenueFiltersDto } from 'src/types/dashboardFilters';
import type { ClientSummaryDto } from 'src/types/clients';
import type { ProjectSummaryDto } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import DashboardDateFilter from './DashboardDateFilter.vue';
import MonthlyRevenueChart from './MonthlyRevenueChart.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  palette: ChartPalette;
  emptyLabel: string;
  initialRows: MonthlyRevenueRow[];
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const currencyOptions = CURRENCY_TYPE_OPTIONS;

const defaults = (): MonthlyRevenueFiltersDto => ({
  date_from: null,
  date_to: null,
  currency_type: null,
  client_id: null,
  project_id: null,
});

const state = useChartCardState<MonthlyRevenueFiltersDto, MonthlyRevenueRow>({
  defaults,
  fetchRows: (filters) => DashboardService.getMonthlyRevenue(filters),
  initialRows: props.initialRows,
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
