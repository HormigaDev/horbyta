<template>
  <DashboardChartCardWithSettings
    :title="t('clientDashboard.projectStatus.title')"
    :caption="t('clientDashboard.projectStatus.caption')"
    :filters-title="t('dashboard.filters.clientProjectStatus.title')"
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
    <ClientProjectStatusChart
      :rows="state.rows.value"
      :palette="palette"
      :empty-label="t('clientDashboard.empty')"
      :label-for-status="labelForStatus"
    />

    <template #filters>
      <q-select
        v-model="state.draftFilters.project_status"
        :options="projectStatusOptions"
        :label="t('dashboard.filters.common.projectStatus')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
      <q-select
        v-model="state.draftFilters.billable"
        :options="billableOptions"
        :label="t('dashboard.filters.common.billable')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
    </template>
  </DashboardChartCardWithSettings>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { useChartCardState } from 'src/composables/dashboard/useChartCardState';
import { ClientDashboardService } from 'src/services/clientDashboard';
import type { ChartPalette } from 'src/composables/dashboard/useChartPalette';
import type { ClientProjectStatusRow } from 'src/types/dashboard';
import type { ClientProjectStatusFiltersDto } from 'src/types/dashboardFilters';
import type { ProjectStatus } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import ClientProjectStatusChart from './ClientProjectStatusChart.vue';

interface Props {
  palette: ChartPalette;
  initialRows: ClientProjectStatusRow[];
  labelForStatus: (status: ProjectStatus) => string;
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const projectStatusOptions = computed(() => [
  { label: t('projects.status.open'), value: 'open' },
  { label: t('projects.status.closed'), value: 'closed' },
  { label: t('projects.status.cancelled'), value: 'cancelled' },
]);

const billableOptions = computed(() => [
  { label: t('common.yes'), value: true },
  { label: t('common.no'), value: false },
]);

const defaults = (): ClientProjectStatusFiltersDto => ({
  project_status: null,
  billable: null,
  category_id: null,
});

const state = useChartCardState<ClientProjectStatusFiltersDto, ClientProjectStatusRow>({
  defaults,
  fetchRows: (filters) => ClientDashboardService.getProjectStatus(filters),
  initialRows: () => props.initialRows,
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
