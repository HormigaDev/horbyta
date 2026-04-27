<template>
  <DashboardChartCardWithSettings
    :title="t('clientDashboard.portfolio.title')"
    :caption="t('clientDashboard.portfolio.caption')"
    short
    :filters-title="t('dashboard.filters.clientProjectPortfolio.title')"
    :open="state.dialogOpen.value"
    @open="state.openDialog"
    @close="state.dialogOpen.value = false"
    @apply="onApply"
    @reset="onReset"
  >
    <ClientProjectPortfolioList
      :rows="state.rows.value"
      :empty-label="t('clientDashboard.portfolio.empty')"
      :status-label="t('common.status')"
      :billing-label="t('clientDashboard.portfolio.billing')"
      :next-milestone-label="t('clientDashboard.portfolio.nextMilestone')"
      :total-milestones-label="t('clientDashboard.milestones.total')"
      :completed-label="t('clientDashboard.milestones.completed')"
      :upcoming-label="t('clientDashboard.milestones.upcoming')"
      :overdue-label="t('clientDashboard.milestones.overdue')"
      :empty-value="t('common.empty')"
      :day-label="t('projects.detail.paymentDayValue', { day: '{day}' })"
      :label-for-project-status="labelForProjectStatus"
      :label-for-billing-type="labelForBillingType"
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
      <q-select
        v-model="state.draftFilters.billing_type"
        :options="billingTypeOptions"
        :label="t('dashboard.filters.common.billingType')"
        emit-value
        map-options
        clearable
        outlined
        dense
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
      <q-toggle
        :model-value="state.draftFilters.overdue_milestones_only ?? false"
        :label="t('dashboard.filters.clientProjectPortfolio.overdueMilestonesOnly')"
        @update:model-value="(value) => (state.draftFilters.overdue_milestones_only = value)"
      />
      <q-toggle
        :model-value="state.draftFilters.upcoming_milestones_only ?? false"
        :label="t('dashboard.filters.clientProjectPortfolio.upcomingMilestonesOnly')"
        @update:model-value="(value) => (state.draftFilters.upcoming_milestones_only = value)"
      />
    </template>
  </DashboardChartCardWithSettings>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

import { useUI } from 'src/composables/useUI';
import { useChartCardState } from 'src/composables/dashboard/useChartCardState';
import { CURRENCY_TYPE_OPTIONS } from 'src/data/constants';
import { ClientDashboardService } from 'src/services/clientDashboard';
import type { ClientProjectPortfolioRow } from 'src/types/dashboard';
import type { ClientProjectPortfolioFiltersDto } from 'src/types/dashboardFilters';
import type { ProjectBillingType, ProjectStatus } from 'src/types/project';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import ClientProjectPortfolioList from './ClientProjectPortfolioList.vue';

interface Props {
  initialRows: ClientProjectPortfolioRow[];
  labelForProjectStatus: (status: ProjectStatus) => string;
  labelForBillingType: (billingType: ProjectBillingType) => string;
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const currencyOptions = CURRENCY_TYPE_OPTIONS;

const projectStatusOptions = computed(() => [
  { label: props.labelForProjectStatus('open'), value: 'open' },
  { label: props.labelForProjectStatus('closed'), value: 'closed' },
  { label: props.labelForProjectStatus('cancelled'), value: 'cancelled' },
]);

const billableOptions = computed(() => [
  { label: t('common.yes'), value: true },
  { label: t('common.no'), value: false },
]);

const billingTypeOptions = computed(() => [
  { label: props.labelForBillingType('fixed'), value: 'fixed' },
  { label: props.labelForBillingType('hourly'), value: 'hourly' },
  { label: props.labelForBillingType('retainer'), value: 'retainer' },
]);

const defaults = (): ClientProjectPortfolioFiltersDto => ({
  project_status: null,
  billable: null,
  billing_type: null,
  currency_type: null,
  overdue_milestones_only: null,
  upcoming_milestones_only: null,
});

const state = useChartCardState<ClientProjectPortfolioFiltersDto, ClientProjectPortfolioRow>({
  defaults,
  fetchRows: (filters) => ClientDashboardService.getProjectPortfolio(filters),
  initialRows: props.initialRows,
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
