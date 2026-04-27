<template>
  <DashboardChartCardWithSettings
    :title="t('clientDashboard.changeRequests.title')"
    :caption="t('clientDashboard.changeRequests.caption')"
    short
    :filters-title="t('dashboard.filters.clientChangeRequestStatus.title')"
    :open="state.dialogOpen.value"
    @open="state.openDialog"
    @close="state.dialogOpen.value = false"
    @apply="onApply"
    @reset="onReset"
  >
    <ClientChangeRequestStatusList
      :rows="state.rows.value"
      :empty-label="t('clientDashboard.changeRequests.empty')"
      :label-for-status="labelForStatus"
      :helper-for-status="helperForStatus"
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
        v-model="state.draftFilters.status"
        :options="statusOptions"
        :label="t('dashboard.filters.common.status')"
        emit-value
        map-options
        clearable
        outlined
        dense
      />
      <DashboardDateFilter
        v-model="state.draftFilters.date_from"
        :label="t('dashboard.filters.common.dateFrom')"
      />
      <DashboardDateFilter
        v-model="state.draftFilters.date_to"
        :label="t('dashboard.filters.common.dateTo')"
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
import type { ClientChangeRequestStatusRow } from 'src/types/dashboard';
import type { ClientChangeRequestStatusFiltersDto } from 'src/types/dashboardFilters';
import type { ChangeRequestStatus } from 'src/types/changeRequest';
import type { ClientProjectSummaryDto } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import ClientChangeRequestStatusList from './ClientChangeRequestStatusList.vue';
import DashboardDateFilter from './DashboardDateFilter.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  initialRows: ClientChangeRequestStatusRow[];
  labelForStatus: (status: ChangeRequestStatus) => string;
  helperForStatus: (status: ChangeRequestStatus) => string;
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const statusOptions = computed(() => [
  { label: props.labelForStatus('draft'), value: 'draft' },
  { label: props.labelForStatus('submitted'), value: 'submitted' },
  { label: props.labelForStatus('assessed'), value: 'assessed' },
  { label: props.labelForStatus('approved'), value: 'approved' },
  { label: props.labelForStatus('declined'), value: 'declined' },
  { label: props.labelForStatus('implemented'), value: 'implemented' },
  { label: props.labelForStatus('cancelled'), value: 'cancelled' },
]);

const defaults = (): ClientChangeRequestStatusFiltersDto => ({
  project_id: null,
  status: null,
  date_from: null,
  date_to: null,
});

const state = useChartCardState<ClientChangeRequestStatusFiltersDto, ClientChangeRequestStatusRow>({
  defaults,
  fetchRows: (filters) => ClientDashboardService.getChangeRequestStatus(filters),
  initialRows: props.initialRows,
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
