<template>
  <q-page class="detail-page q-pa-md row items-start q-col-gutter-md">
    <ContentSkeleton v-if="contentLoading" variant="detail" class="col-12 loading-page-region" />
    <template v-else>
      <div class="col-12 col-md-2"></div>

      <div class="col-12 col-md-8">
        <q-card
          id="project-information"
          tabindex="-1"
          class="q-mb-md project-section-anchor"
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section>
            <h1 class="text-h5 text-weight-bold q-ma-none" style="color: var(--text-color)">
              {{ project?.name || t('common.loading') }}
            </h1>
            <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="project">
              {{ t('common.status') }}:
              <span style="color: var(--text-color)">
                {{ projectStatusLabel(project.status) }}
              </span>
            </div>
          </q-card-section>
        </q-card>

        <ScopeProjectPanel
          ref="scopePanel"
          v-if="project"
          :key="project.id"
          :project-id="project.id"
          client
        >
          <template #summary>
            <section class="project-summary-meta">
              <div class="text-caption" style="color: var(--text-muted)" v-if="project">
                {{
                  t('projects.detail.meta', {
                    id: project.id,
                    created: formatDate(project.created_at),
                    updated: formatDate(project.updated_at),
                  })
                }}
              </div>
            </section>

            <q-card
              id="project-billing"
              tabindex="-1"
              class="q-mt-md project-section-anchor"
              flat
              bordered
              style="background-color: var(--card-bg); border-color: var(--border-color)"
            >
              <q-card-section>
                <h2
                  class="text-subtitle1 q-mb-md q-mt-none text-weight-bold"
                  style="color: var(--text-accent)"
                >
                  {{ t('projects.detail.billingTitle') }}
                </h2>

                <div class="detail-content">
                  <div
                    v-if="project && project.billable && project.project_billing"
                    class="row q-col-gutter-md"
                  >
                    <div class="col-12 col-md-3">
                      <div class="text-caption" style="color: var(--text-muted)">
                        {{ t('projects.detail.currencyTypeLabel') }}
                      </div>
                      <div style="color: var(--text-color)">
                        {{ project.project_billing.currency_type }}
                      </div>
                    </div>

                    <div class="col-12 col-md-3">
                      <div class="text-caption" style="color: var(--text-muted)">
                        {{ t('projects.detail.billingTypeLabel') }}
                      </div>
                      <div style="color: var(--text-color)">
                        {{ projectBillingTypeLabel(project.project_billing.billing_type) }}
                      </div>
                    </div>

                    <div
                      class="col-12 col-md-3"
                      v-if="project.project_billing.billing_type === 'fixed'"
                    >
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

                    <div
                      class="col-12 col-md-3"
                      v-if="project.project_billing.billing_type === 'hourly'"
                    >
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
                      <div class="col-12 col-md-3">
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

                      <div class="col-12 col-md-3">
                        <div class="text-caption" style="color: var(--text-muted)">
                          {{ t('projects.detail.paymentDayLabel') }}
                        </div>
                        <div style="color: var(--text-color)">
                          {{ paymentDayLabel(project.project_billing.payment_day) }}
                        </div>
                      </div>
                    </template>
                  </div>

                  <div
                    v-else-if="project && !project.billable"
                    class="text-h5 text-weight-bold"
                    style="color: var(--text-color)"
                  >
                    {{ t('projects.detail.noCost') }}
                  </div>
                  <div v-else class="text-caption" style="color: var(--text-muted)">
                    {{ t('projects.detail.billingEmpty') }}
                  </div>
                </div>
              </q-card-section>
            </q-card>

            <q-card
              id="project-description"
              tabindex="-1"
              class="project-section-anchor"
              flat
              bordered
              style="background-color: var(--card-bg); border-color: var(--border-color)"
            >
              <q-card-section>
                <h2
                  class="text-subtitle1 q-mb-md q-mt-none text-weight-bold"
                  style="color: var(--text-accent)"
                >
                  {{ t('projects.detail.descriptionTitle') }}
                </h2>
                <div class="detail-content">
                  <MarkdownRenderer
                    :model-value="project?.description || t('projects.detail.emptyDescription')"
                  />
                </div>
              </q-card-section>
            </q-card>
          </template>
        </ScopeProjectPanel>
      </div>

      <div class="col-12 col-md-2" style="position: sticky; top: 80px">
        <q-card
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section class="project-sidebar-heading">{{
            t('projects.detail.quickActionsTitle')
          }}</q-card-section>
          <q-list separator>
            <q-item clickable v-ripple @click="goBack" class="q-pa-md">
              <q-item-section avatar>
                <q-icon name="arrow_back" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('projects.detail.action.back') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="project"
              clickable
              v-ripple
              class="q-pa-md"
              @click="scopePanel?.focusConversation()"
            >
              <q-item-section avatar><q-icon name="forum" /></q-item-section>
              <q-item-section
                >{{ t('scope.conversationAction') }}
                <ConversationUnreadBadge :project-id="project.id" class="q-ml-sm"
              /></q-item-section>
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
        <q-card
          v-if="project"
          flat
          bordered
          class="q-mt-md"
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section class="project-sidebar-heading">{{
            t('projects.detail.indexTitle')
          }}</q-card-section>
          <nav :aria-label="t('projects.detail.indexTitle')" class="project-section-index">
            <q-list dense>
              <q-item
                v-for="entry in sectionIndex"
                :key="entry.id"
                clickable
                v-ripple
                tag="a"
                :href="`#${entry.id}`"
                @click.prevent="scopePanel?.focusSection(entry.id)"
              >
                <q-item-section>{{ entry.label }}</q-item-section>
              </q-item>
            </q-list>
          </nav>
        </q-card>
      </div>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import ConversationUnreadBadge from 'src/components/ConversationUnreadBadge.vue';
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
const contentLoading = ref(true);
import ScopeProjectPanel from 'src/components/ScopeProjectPanel.vue';
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { ClientProjectsService } from 'src/services/clientProjects';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ClientProjectDetailDto, ProjectBillingType, ProjectStatus } from 'src/types/project';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notifyError } = useUI();
const userPreferencesStore = useDisplayPreferences();
const { formatCurrency } = useNumberFormatter();

const scopePanel = ref<InstanceType<typeof ScopeProjectPanel> | null>(null);
const project = ref<ClientProjectDetailDto | null>(null);
const projectId = String(route.params.id || '');
const sectionIndex = computed(() => [
  { id: 'project-information', label: t('projects.detail.basicInfoTitle') },
  { id: `scope-project-${projectId}-scope`, label: t('scope.title') },
  ...(scopePanel.value?.sectionIndex ?? []),
  { id: 'project-billing', label: t('projects.detail.billingTitle') },
  { id: 'project-description', label: t('projects.detail.descriptionTitle') },
]);

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
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

  contentLoading.value = true;
  try {
    project.value = await ClientProjectsService.getById(projectId);
  } catch (err: unknown) {
    notifyError(t('projects.detail.loadError'), err);
    void router.push('/client/projects');
  } finally {
    contentLoading.value = false;
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

<style scoped>
.project-sidebar-heading {
  padding: 12px 16px 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-muted);
}
.project-section-index {
  max-height: 40vh;
  overflow-y: auto;
  padding-bottom: 8px;
}
.project-section-index .q-item {
  font-size: 13px;
  padding: 8px 16px;
}
.project-section-anchor {
  scroll-margin-top: 80px;
}
</style>
