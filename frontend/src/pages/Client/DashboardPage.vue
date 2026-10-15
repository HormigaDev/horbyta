<template>
  <q-page class="q-pa-md dashboard-page">
    <PageHeader
      :title="t('clientDashboard.title')"
      :subtitle="t('clientDashboard.subtitle')"
      :eyebrow="t('ui.workspace')"
    >
      <template #actions
        ><q-btn
          outline
          color="primary"
          icon="refresh"
          :label="t('clientDashboard.refresh')"
          :loading="isLoading"
          @click="fetchDashboard"
      /></template>
    </PageHeader>

    <ScopePendingPanel />
    <ContentSkeleton v-if="isLoading && !data" variant="dashboard" />

    <EmptyState v-if="loadError" error :title="t('clientDashboard.error')"
      ><q-btn outline color="primary" :label="t('ui.retry')" @click="fetchDashboard"
    /></EmptyState>

    <template v-if="data">
      <div class="row q-col-gutter-md">
        <div class="col-12 col-sm-6 col-md-3">
          <DashboardKpiCard
            :title="t('clientDashboard.kpi.activeProjects.title')"
            :caption="t('clientDashboard.kpi.activeProjects.caption')"
            action-to="/client/projects"
            :action-label="t('layout.drawer.projects')"
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
            action-to="/client/milestones"
            :action-label="t('layout.drawer.milestones')"
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
            action-to="/client/change-requests"
            :action-label="t('layout.drawer.changeRequests')"
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

      <div class="dashboard-section-heading">{{ t('ui.overview') }}</div>
      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-5">
          <ClientProjectStatusCard
            :initial-rows="data.project_status"
            :palette="chartPalette"
            :label-for-status="projectStatusLabel"
          />
        </div>

        <div class="col-12 col-md-7">
          <ClientChangeRequestStatusCard
            :initial-rows="data.change_request_status"
            :label-for-status="changeRequestStatusLabel"
            :helper-for-status="changeRequestStatusHelper"
          />
        </div>
      </div>

      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-md-6">
          <ClientMilestoneHealthCard :initial-rows="data.milestone_health" />
        </div>

        <div class="col-12 col-md-6">
          <ClientProjectPortfolioCard
            :initial-rows="data.project_portfolio"
            :label-for-project-status="projectStatusLabel"
            :label-for-billing-type="projectBillingTypeLabel"
          />
        </div>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
import ScopePendingPanel from 'src/components/ScopePendingPanel.vue';
import PageHeader from 'src/components/PageHeader.vue';
import EmptyState from 'src/components/EmptyState.vue';
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';

import type { ChangeRequestStatus } from 'src/types/changeRequest';
import type { ClientDashboardDto } from 'src/types/dashboard';
import type { ProjectBillingType, ProjectStatus } from 'src/types/project';
import { ClientDashboardService } from 'src/services/clientDashboard';
import { useUI } from 'src/composables/useUI';
import { useChartPalette } from 'src/composables/dashboard/useChartPalette';
import DashboardKpiCard from 'src/components/dashboard/DashboardKpiCard.vue';
import CurrencyAmountValue from 'src/components/dashboard/CurrencyAmountValue.vue';
import ClientProjectStatusCard from 'src/components/dashboard/ClientProjectStatusCard.vue';
import ClientMilestoneHealthCard from 'src/components/dashboard/ClientMilestoneHealthCard.vue';
import ClientChangeRequestStatusCard from 'src/components/dashboard/ClientChangeRequestStatusCard.vue';
import ClientProjectPortfolioCard from 'src/components/dashboard/ClientProjectPortfolioCard.vue';

const { t } = useI18n();
const { notifyError } = useUI();
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

const changeRequestStatusHelper = (status: ChangeRequestStatus) =>
  changeRequestStatusHelperMap.value[status];

onMounted(() => {
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
