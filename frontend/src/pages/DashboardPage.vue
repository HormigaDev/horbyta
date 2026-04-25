<template>
  <q-page class="q-pa-md dashboard-page">
    <div class="dashboard-header q-mb-md">
      <div>
        <div class="text-h5 text-weight-bold">{{ t('dashboard.title') }}</div>
        <div class="text-caption text-muted">{{ t('dashboard.subtitle') }}</div>
      </div>
      <q-btn
        flat
        dense
        icon="refresh"
        :label="t('dashboard.refresh')"
        :loading="isLoading"
        @click="fetchDashboard"
      />
    </div>

    <div v-if="isLoading && !data" class="flex flex-center q-py-xl">
      <q-spinner color="primary" size="32px" />
      <span class="q-ml-sm text-muted">{{ t('dashboard.loading') }}</span>
    </div>

    <div v-else-if="loadError" class="text-center q-py-xl text-negative">
      {{ t('dashboard.error') }}
    </div>

    <template v-else-if="data">
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

      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-8">
          <DashboardChartCard
            :title="t('dashboard.charts.monthlyRevenue.title')"
            :caption="t('dashboard.charts.monthlyRevenue.caption')"
          >
            <MonthlyRevenueChart
              :rows="data.monthly_revenue"
              :palette="chartPalette"
              :empty-label="t('dashboard.empty')"
            />
          </DashboardChartCard>
        </div>

        <div class="col-12 col-md-4">
          <DashboardChartCard
            :title="t('dashboard.charts.timeByActivity.title')"
            :caption="t('dashboard.charts.timeByActivity.caption')"
          >
            <TimeByActivityChart
              :rows="data.time_by_activity"
              :palette="chartPalette"
              :empty-label="t('dashboard.empty')"
            />
          </DashboardChartCard>
        </div>

        <div class="col-12">
          <DashboardChartCard
            :title="t('dashboard.charts.billableRatio.title')"
            :caption="t('dashboard.charts.billableRatio.caption')"
            short
          >
            <BillableRatioChart
              :rows="data.billable_ratio_daily"
              :palette="chartPalette"
              :empty-label="t('dashboard.empty')"
              :billable-label="t('dashboard.charts.series.billable')"
              :non-billable-label="t('dashboard.charts.series.nonBillable')"
            />
          </DashboardChartCard>
        </div>
      </div>

      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-7">
          <DashboardChartCard
            :title="t('dashboard.bottlenecks.title')"
            :caption="t('dashboard.bottlenecks.caption')"
            short
          >
            <TaskBottlenecksList
              :rows="data.task_bottlenecks"
              :labels="bottleneckLabels"
              :empty-label="t('dashboard.bottlenecks.empty')"
            />
          </DashboardChartCard>
        </div>

        <div class="col-12 col-md-5">
          <DashboardChartCard
            :title="t('dashboard.accountsReceivable.title')"
            :caption="t('dashboard.kpi.accountsReceivable.caption')"
            short
          >
            <AccountsReceivableList
              :rows="data.accounts_receivable"
              :empty-label="t('dashboard.accountsReceivable.empty')"
              :oldest-due-date-label="t('dashboard.accountsReceivable.oldestDueDate')"
            />
          </DashboardChartCard>
        </div>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';

import { DashboardService } from 'src/services/dashboard';
import { useUI } from 'src/composables/useUI';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { DashboardDto } from 'src/types/dashboard';
import DashboardKpiCard from 'src/components/dashboard/DashboardKpiCard.vue';
import DashboardChartCard from 'src/components/dashboard/DashboardChartCard.vue';
import CurrencyAmountValue from 'src/components/dashboard/CurrencyAmountValue.vue';
import MonthlyRevenueChart from 'src/components/dashboard/MonthlyRevenueChart.vue';
import TimeByActivityChart from 'src/components/dashboard/TimeByActivityChart.vue';
import BillableRatioChart from 'src/components/dashboard/BillableRatioChart.vue';
import TaskBottlenecksList, {
  type TaskBottlenecksLabels,
} from 'src/components/dashboard/TaskBottlenecksList.vue';
import AccountsReceivableList from 'src/components/dashboard/AccountsReceivableList.vue';
import { useChartPalette } from 'src/composables/dashboard/useChartPalette';

const { t } = useI18n();
const { notifyError } = useUI();
const preferencesStore = useUserPreferencesStore();
const chartPalette = useChartPalette();

const data = ref<DashboardDto | null>(null);
const isLoading = ref(false);
const loadError = ref(false);

const bottleneckLabels = computed<TaskBottlenecksLabels>(() => ({
  total: t('dashboard.bottlenecks.column.total'),
  todo: t('dashboard.bottlenecks.column.todo'),
  inProgress: t('dashboard.bottlenecks.column.inProgress'),
  review: t('dashboard.bottlenecks.column.review'),
  overdue: t('dashboard.bottlenecks.column.overdue'),
}));

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
