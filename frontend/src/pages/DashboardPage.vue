<template>
  <q-page class="q-pa-md dashboard-page">
    <PageHeader
      :title="t('dashboard.title')"
      :subtitle="t('dashboard.subtitle')"
      :eyebrow="t('ui.workspace')"
    >
      <template #actions
        ><q-btn
          outline
          color="primary"
          icon="refresh"
          :label="t('dashboard.refresh')"
          :loading="isLoading"
          @click="fetchDashboard"
      /></template>
    </PageHeader>

    <div v-if="isLoading && !data" class="flex flex-center q-py-xl">
      <q-spinner color="primary" size="32px" />
      <span class="q-ml-sm text-muted">{{ t('dashboard.loading') }}</span>
    </div>

    <EmptyState v-if="loadError" error :title="t('dashboard.error')"
      ><q-btn outline color="primary" :label="t('ui.retry')" @click="fetchDashboard"
    /></EmptyState>

    <template v-if="data">
      <div class="row q-col-gutter-md">
        <div class="col-12 col-sm-6 col-md-4">
          <DashboardKpiCard
            :title="t('dashboard.kpi.scopeCreep.title')"
            :caption="t('dashboard.kpi.scopeCreep.caption')"
            icon="trending_up"
            variant="positive"
          >
            <template #values>
              <template v-if="data.kpis.scope_creep_recovered.length > 0">
                <CurrencyAmountValue
                  v-for="item in data.kpis.scope_creep_recovered"
                  :key="`sc-${item.currency_type}`"
                  :amount="item.amount"
                  :currency-type="item.currency_type"
                />
              </template>
              <div v-else class="kpi-empty">—</div>
            </template>
          </DashboardKpiCard>
        </div>

        <div class="col-12 col-sm-6 col-md-4">
          <DashboardKpiCard
            :title="t('dashboard.kpi.accountsReceivable.title')"
            :caption="t('dashboard.kpi.accountsReceivable.caption')"
            action-to="/invoices"
            :action-label="t('layout.drawer.invoices')"
            icon="request_quote"
            variant="warning"
          >
            <template #values>
              <template v-if="data.kpis.accounts_receivable_total.length > 0">
                <CurrencyAmountValue
                  v-for="item in data.kpis.accounts_receivable_total"
                  :key="`ar-${item.currency_type}`"
                  :amount="item.amount"
                  :currency-type="item.currency_type"
                />
              </template>
              <div v-else class="kpi-empty">—</div>
            </template>
          </DashboardKpiCard>
        </div>

        <div class="col-12 col-sm-12 col-md-4">
          <DashboardKpiCard
            :title="t('dashboard.kpi.overdueTasks.title')"
            :caption="t('dashboard.kpi.overdueTasks.caption')"
            action-to="/tasks"
            :action-label="t('layout.drawer.tasks')"
            icon="schedule"
            :variant="data.kpis.overdue_tasks_total > 0 ? 'negative' : 'muted'"
          >
            <template #values>
              <div class="kpi-value">
                <span class="kpi-amount kpi-amount--big">
                  {{ data.kpis.overdue_tasks_total }}
                </span>
              </div>
            </template>
          </DashboardKpiCard>
        </div>
      </div>

      <div class="dashboard-section-heading">{{ t('ui.trends') }}</div>
      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-8">
          <MonthlyRevenueCard
            :initial-rows="data.monthly_revenue"
            :palette="chartPalette"
            :empty-label="t('dashboard.empty')"
          />
        </div>

        <div class="col-12 col-md-4">
          <TimeByActivityCard
            :initial-rows="data.time_by_activity"
            :palette="chartPalette"
            :empty-label="t('dashboard.empty')"
          />
        </div>

        <div class="col-12">
          <BillableRatioCard
            :initial-rows="data.billable_ratio_daily"
            :palette="chartPalette"
            :empty-label="t('dashboard.empty')"
          />
        </div>
      </div>

      <div class="dashboard-section-heading">{{ t('ui.attention') }}</div>
      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-7">
          <TaskBottlenecksCard :initial-rows="data.task_bottlenecks" />
        </div>

        <div class="col-12 col-md-5">
          <AccountsReceivableCard :initial-rows="data.accounts_receivable" />
        </div>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import PageHeader from 'src/components/PageHeader.vue';
import EmptyState from 'src/components/EmptyState.vue';
import { defineAsyncComponent, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';

import { DashboardService } from 'src/services/dashboard';
import { useUI } from 'src/composables/useUI';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { DashboardDto } from 'src/types/dashboard';
import DashboardKpiCard from 'src/components/dashboard/DashboardKpiCard.vue';
import CurrencyAmountValue from 'src/components/dashboard/CurrencyAmountValue.vue';
import { useChartPalette } from 'src/composables/dashboard/useChartPalette';

const MonthlyRevenueCard = defineAsyncComponent(
  () => import('src/components/dashboard/MonthlyRevenueCard.vue'),
);
const TimeByActivityCard = defineAsyncComponent(
  () => import('src/components/dashboard/TimeByActivityCard.vue'),
);
const BillableRatioCard = defineAsyncComponent(
  () => import('src/components/dashboard/BillableRatioCard.vue'),
);
const TaskBottlenecksCard = defineAsyncComponent(
  () => import('src/components/dashboard/TaskBottlenecksCard.vue'),
);
const AccountsReceivableCard = defineAsyncComponent(
  () => import('src/components/dashboard/AccountsReceivableCard.vue'),
);

const { t } = useI18n();
const { notifyError } = useUI();
const preferencesStore = useUserPreferencesStore();
const chartPalette = useChartPalette();

const data = ref<DashboardDto | null>(null);
const isLoading = ref(false);
const loadError = ref(false);

const fetchDashboard = async () => {
  isLoading.value = true;
  loadError.value = false;
  try {
    data.value = await DashboardService.getOverview();
  } catch (error) {
    loadError.value = true;
    notifyError(t('dashboard.error'), error);
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  void preferencesStore.ensureLoaded();
  void fetchDashboard();
});
</script>

<style scoped lang="scss">
.dashboard-page {
  min-height: 100%;
}

.dashboard-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.text-muted {
  color: var(--text-muted);
}

.kpi-value {
  display: flex;
  align-items: baseline;
  gap: 6px;
  color: var(--text-color);
}

.kpi-amount {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-color);
}

.kpi-amount--big {
  font-size: 32px;
}

.kpi-empty {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-muted);
}

@media (max-width: 600px) {
  .kpi-amount {
    font-size: 20px;
  }
  .kpi-amount--big {
    font-size: 28px;
  }
}
</style>
