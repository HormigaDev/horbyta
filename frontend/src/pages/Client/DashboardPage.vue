<template>
  <q-page class="q-pa-md dashboard-page">
    <div class="dashboard-header q-mb-md">
      <div>
        <div class="text-h5 text-weight-bold">{{ t('clientDashboard.title') }}</div>
        <div class="text-caption text-muted">{{ t('clientDashboard.subtitle') }}</div>
      </div>
      <q-btn
        flat
        dense
        icon="refresh"
        :label="t('clientDashboard.refresh')"
        :loading="isLoading"
        @click="fetchDashboard"
      />
    </div>

    <div v-if="isLoading && !data" class="flex flex-center q-py-xl">
      <q-spinner color="primary" size="32px" />
      <span class="q-ml-sm text-muted">{{ t('clientDashboard.loading') }}</span>
    </div>

    <div v-else-if="loadError" class="text-center q-py-xl text-negative">
      {{ t('clientDashboard.error') }}
    </div>

    <template v-else-if="data">
      <div class="row q-col-gutter-md">
        <div class="col-12 col-sm-6 col-md-3">
          <DashboardKpiCard
            :title="t('clientDashboard.kpi.activeProjects.title')"
            :caption="t('clientDashboard.kpi.activeProjects.caption')"
            icon="folder_open"
            variant="positive"
          >
            <template #values>
              <div class="kpi-value">
                <span class="kpi-amount kpi-amount--big">
                  {{ data.kpis.active_projects_total }}
                </span>
              </div>
            </template>
          </DashboardKpiCard>
        </div>

        <div class="col-12 col-sm-6 col-md-3">
          <DashboardKpiCard
            :title="t('clientDashboard.kpi.overdueMilestones.title')"
            :caption="t('clientDashboard.kpi.overdueMilestones.caption')"
            icon="event_busy"
            :variant="data.kpis.overdue_milestones_total > 0 ? 'negative' : 'muted'"
          >
            <template #values>
              <div class="kpi-value">
                <span class="kpi-amount kpi-amount--big">
                  {{ data.kpis.overdue_milestones_total }}
                </span>
              </div>
            </template>
          </DashboardKpiCard>
        </div>

        <div class="col-12 col-sm-6 col-md-3">
          <DashboardKpiCard
            :title="t('clientDashboard.kpi.openChangeRequests.title')"
            :caption="t('clientDashboard.kpi.openChangeRequests.caption')"
            icon="compare_arrows"
            variant="warning"
          >
            <template #values>
              <div class="kpi-value">
                <span class="kpi-amount kpi-amount--big">
                  {{ data.kpis.open_change_requests_total }}
                </span>
              </div>
            </template>
          </DashboardKpiCard>
        </div>

        <div class="col-12 col-sm-6 col-md-3">
          <DashboardKpiCard
            :title="t('clientDashboard.kpi.approvedChangeValue.title')"
            :caption="t('clientDashboard.kpi.approvedChangeValue.caption')"
            icon="payments"
            variant="default"
          >
            <template #values>
              <template v-if="data.kpis.approved_change_value.length > 0">
                <CurrencyAmountValue
                  v-for="item in data.kpis.approved_change_value"
                  :key="`cv-${item.currency_type}`"
                  :amount="item.amount"
                  :currency-type="item.currency_type"
                />
              </template>
              <div v-else class="kpi-empty">—</div>
            </template>
          </DashboardKpiCard>
        </div>
      </div>

      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-5">
          <DashboardChartCard
            :title="t('clientDashboard.projectStatus.title')"
            :caption="t('clientDashboard.projectStatus.caption')"
          >
            <ClientProjectStatusChart
              :rows="data.project_status"
              :palette="chartPalette"
              :empty-label="t('clientDashboard.empty')"
              :label-for-status="projectStatusLabel"
            />
          </DashboardChartCard>
        </div>

        <div class="col-12 col-md-7">
          <DashboardChartCard
            :title="t('clientDashboard.changeRequests.title')"
            :caption="t('clientDashboard.changeRequests.caption')"
            short
          >
            <ClientChangeRequestStatusList
              :rows="data.change_request_status"
              :empty-label="t('clientDashboard.changeRequests.empty')"
              :label-for-status="changeRequestStatusLabel"
              :helper-for-status="changeRequestStatusHelper"
            />
          </DashboardChartCard>
        </div>
      </div>

      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-6">
          <DashboardChartCard
            :title="t('clientDashboard.milestones.title')"
            :caption="t('clientDashboard.milestones.caption')"
            short
          >
            <ClientMilestoneHealthList
              :rows="data.milestone_health"
              :empty-label="t('clientDashboard.milestones.empty')"
              :total-label="t('clientDashboard.milestones.total')"
              :completed-label="t('clientDashboard.milestones.completed')"
              :upcoming-label="t('clientDashboard.milestones.upcoming')"
              :overdue-label="t('clientDashboard.milestones.overdue')"
              :next-due-date-label="t('clientDashboard.milestones.nextDueDate')"
              :empty-value="t('common.empty')"
            />
          </DashboardChartCard>
        </div>

        <div class="col-12 col-md-6">
          <DashboardChartCard
            :title="t('clientDashboard.portfolio.title')"
            :caption="t('clientDashboard.portfolio.caption')"
            short
          >
            <ClientProjectPortfolioList
              :rows="data.project_portfolio"
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
              :label-for-project-status="projectStatusLabel"
              :label-for-billing-type="projectBillingTypeLabel"
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

import type { ChangeRequestStatus } from 'src/types/changeRequest';
import type { ClientDashboardDto } from 'src/types/dashboard';
import type { ProjectBillingType, ProjectStatus } from 'src/types/project';
import { ClientDashboardService } from 'src/services/clientDashboard';
import { useUI } from 'src/composables/useUI';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useChartPalette } from 'src/composables/dashboard/useChartPalette';
import DashboardKpiCard from 'src/components/dashboard/DashboardKpiCard.vue';
import DashboardChartCard from 'src/components/dashboard/DashboardChartCard.vue';
import CurrencyAmountValue from 'src/components/dashboard/CurrencyAmountValue.vue';
import ClientProjectStatusChart from 'src/components/dashboard/ClientProjectStatusChart.vue';
import ClientMilestoneHealthList from 'src/components/dashboard/ClientMilestoneHealthList.vue';
import ClientChangeRequestStatusList from 'src/components/dashboard/ClientChangeRequestStatusList.vue';
import ClientProjectPortfolioList from 'src/components/dashboard/ClientProjectPortfolioList.vue';

const { t } = useI18n();
const { notifyError } = useUI();
const preferencesStore = useUserPreferencesStore();
const chartPalette = useChartPalette();

const data = ref<ClientDashboardDto | null>(null);
const isLoading = ref(false);
const loadError = ref(false);

const fetchDashboard = async () => {
  isLoading.value = true;
  loadError.value = false;
  try {
    data.value = await ClientDashboardService.getOverview();
  } catch (error) {
    loadError.value = true;
    notifyError(t('clientDashboard.error'), error);
  } finally {
    isLoading.value = false;
  }
};

const projectStatusLabel = (status: ProjectStatus) => t(`projects.status.${status}`);

const projectBillingTypeLabel = (billingType: ProjectBillingType) =>
  t(`projects.billing.type.${billingType}`);

const changeRequestStatusLabel = (status: ChangeRequestStatus) =>
  t(`changeRequests.status.${status}`);

const changeRequestStatusHelperMap = computed<Record<ChangeRequestStatus, string>>(() => ({
  draft: t('clientDashboard.changeRequests.helper.draft'),
  submitted: t('clientDashboard.changeRequests.helper.submitted'),
  assessed: t('clientDashboard.changeRequests.helper.assessed'),
  approved: t('clientDashboard.changeRequests.helper.approved'),
  declined: t('clientDashboard.changeRequests.helper.declined'),
  implemented: t('clientDashboard.changeRequests.helper.implemented'),
  cancelled: t('clientDashboard.changeRequests.helper.cancelled'),
}));

const changeRequestStatusHelper = (status: ChangeRequestStatus) => changeRequestStatusHelperMap.value[status];

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
</style>
