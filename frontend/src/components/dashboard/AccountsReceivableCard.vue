<template>
  <DashboardChartCardWithSettings
    :title="t('dashboard.accountsReceivable.title')"
    :caption="t('dashboard.kpi.accountsReceivable.caption')"
    short
    auto-height
    :filters-title="t('dashboard.filters.accountsReceivable.title')"
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
    <AccountsReceivableList
      :rows="state.rows.value"
      :empty-label="t('dashboard.accountsReceivable.empty')"
      :oldest-due-date-label="t('dashboard.accountsReceivable.oldestDueDate')"
    />

    <template #filters>
      <SelectByTerm
        v-model="state.draftFilters.client_id"
        :options="clientOptions"
        :loading="isClientsLoading"
        :label="t('dashboard.filters.common.client')"
        @search="(term) => loadClients(term)"
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
      <DashboardDateFilter
        v-model="state.draftFilters.due_date_from"
        :label="t('dashboard.filters.accountsReceivable.dueDateFrom')"
      />
      <DashboardDateFilter
        v-model="state.draftFilters.due_date_to"
        :label="t('dashboard.filters.accountsReceivable.dueDateTo')"
      />
      <q-toggle
        :model-value="state.draftFilters.overdue_only ?? false"
        :label="t('dashboard.filters.accountsReceivable.overdueOnly')"
        @update:model-value="(value) => (state.draftFilters.overdue_only = value)"
      />
      <q-input
        v-model="state.draftFilters.min_outstanding_balance"
        outlined
        dense
        type="number"
        clearable
        :label="t('dashboard.filters.accountsReceivable.minOutstanding')"
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
import { DashboardService } from 'src/services/dashboard';
import { CURRENCY_TYPE_OPTIONS } from 'src/data/constants';
import type { AccountsReceivableRow } from 'src/types/dashboard';
import type { AccountsReceivableFiltersDto } from 'src/types/dashboardFilters';
import type { ClientSummaryDto } from 'src/types/clients';
import DashboardChartCardWithSettings from './DashboardChartCardWithSettings.vue';
import DashboardDateFilter from './DashboardDateFilter.vue';
import AccountsReceivableList from './AccountsReceivableList.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';

interface Props {
  initialRows: AccountsReceivableRow[];
}

const props = defineProps<Props>();

const { t } = useI18n();
const { notifyError } = useUI();

const currencyOptions = CURRENCY_TYPE_OPTIONS;

const defaults = (): AccountsReceivableFiltersDto => ({
  client_id: null,
  currency_type: null,
  due_date_from: null,
  due_date_to: null,
  overdue_only: null,
  min_outstanding_balance: null,
});

const state = useChartCardState<AccountsReceivableFiltersDto, AccountsReceivableRow>({
  defaults,
  fetchRows: (filters) => DashboardService.getAccountsReceivable(filters),
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
