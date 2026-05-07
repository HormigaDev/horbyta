<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>

    <div class="col-12 col-md-8">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{ project?.name || t('common.loading') }}
          </div>
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="project">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">
              {{ projectStatusLabel(project.status) }}
            </span>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            {{ t('projects.detail.billableLabel') }}:
            <span style="color: var(--text-color)">
              {{ project.billable ? t('common.yes') : t('common.no') }}
            </span>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="project">
            {{
              t('projects.detail.meta', {
                id: project.id,
                created: formatDate(project.created_at),
                updated: formatDate(project.updated_at),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mt-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('projects.detail.billingTitle') }}
          </div>

          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <div
              v-if="project && project.billable && project.project_billing"
              class="row q-col-gutter-md"
            >
              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.currencyTypeLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ project.project_billing.currency_type }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.billingTypeLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{ projectBillingTypeLabel(project.project_billing.billing_type) }}
                </div>
              </div>

              <div class="col-12 col-md-4" v-if="project.project_billing.billing_type === 'fixed'">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.fixedPriceLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{
                    formatBillingAmount(
                      project.project_billing.fixed_price,
                      project.project_billing.currency_type,
                    )
                  }}
                </div>
              </div>

              <div class="col-12 col-md-4" v-if="project.project_billing.billing_type === 'hourly'">
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('projects.detail.hourlyRateLabel') }}
                </div>
                <div style="color: var(--text-color)">
                  {{
                    formatBillingAmount(
                      project.project_billing.hourly_rate,
                      project.project_billing.currency_type,
                    )
                  }}
                </div>
              </div>

              <template v-if="project.project_billing.billing_type === 'retainer'">
                <div class="col-12 col-md-4">
                  <div class="text-caption" style="color: var(--text-muted)">
                    {{ t('projects.detail.retainerAmountLabel') }}
                  </div>
                  <div style="color: var(--text-color)">
                    {{
                      formatBillingAmount(
                        project.project_billing.retainer_amount,
                        project.project_billing.currency_type,
                      )
                    }}
                  </div>
                </div>

                <div class="col-12 col-md-4">
                  <div class="text-caption" style="color: var(--text-muted)">
                    {{ t('projects.detail.paymentDayLabel') }}
                  </div>
                  <div style="color: var(--text-color)">
                    {{ paymentDayLabel(project.project_billing.payment_day) }}
                  </div>
                </div>
              </template>
            </div>

            <div v-else class="text-caption" style="color: var(--text-muted)">
              {{ t('projects.detail.billingEmpty') }}
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('projects.detail.descriptionTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="project?.description || t('projects.detail.emptyDescription')"
            />
          </div>
        </q-card-section>
      </q-card>
    </div>

    <div class="col-12 col-md-2" style="position: sticky; top: 80px">
      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list separator>
          <q-item clickable v-ripple @click="goBack" class="q-pa-md">
            <q-item-section avatar>
              <q-icon name="arrow_back" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('projects.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item clickable v-ripple class="q-pa-md" @click="goToMilestones">
            <q-item-section avatar>
              <q-icon name="flag" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('projects.detail.action.viewMilestones') }}
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { ClientProjectsService } from 'src/services/clientProjects';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ClientProjectDetailDto, ProjectBillingType, ProjectStatus } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notifyError, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { formatCurrency } = useNumberFormatter();

const project = ref<ClientProjectDetailDto | null>(null);
const projectId = String(route.params.id || '');

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const projectStatusLabel = (status: ProjectStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`projects.status.${status}`);
};

const projectBillingTypeLabel = (billingType: ProjectBillingType | null | undefined) => {
  if (!billingType) return t('common.empty');
  return t(`projects.billing.type.${billingType}`);
};

const formatBillingAmount = (
  amount: string | null | undefined,
  currencyType: string | null | undefined,
) => {
  if (!amount) return t('common.empty');

  return formatCurrency(amount, currencyType, amount, false);
};

const paymentDayLabel = (day: number | null | undefined) => {
  if (day === null || day === undefined) return t('common.empty');
  return t('projects.detail.paymentDayValue', { day });
};

const loadProject = async () => {
  if (!projectId) {
    void router.push('/client/projects');
    return;
  }

  showLoading(t('projects.detail.loadProject'));
  try {
    project.value = await ClientProjectsService.getById(projectId);
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
    void router.push('/client/projects');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/client/projects');
};

const goToMilestones = () => {
  void router.push({
    path: '/client/milestones',
    query: { project_id: projectId },
  });
};

onMounted(() => {
  void loadProject();
});
</script>
