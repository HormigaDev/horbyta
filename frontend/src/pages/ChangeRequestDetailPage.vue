<template>
  <q-page class="detail-page q-pa-md row items-start q-col-gutter-md">
    <ContentSkeleton v-if="contentLoading" variant="detail" class="col-12 loading-page-region" />
    <template v-else>
      <div class="col-12 col-md-2"></div>

      <div class="col-12 col-md-8">
        <q-card
          class="q-mb-md"
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-card-section>
            <div class="flex justify-between">
              <div class="text-h5 text-weight-bold q-mb-xs" style="color: var(--text-color)">
                <q-icon name="compare_arrows" size="lg" />
                {{ changeRequest?.title || t('common.loading') }}
              </div>
              <div class="text-h6" style="color: var(--text-muted)">
                {{ changeRequest ? normalizeId(changeRequest.record_number) : '' }}
              </div>
            </div>
            <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="changeRequest">
              {{ t('common.status') }}:
              <span style="color: var(--text-color)">
                {{ statusLabel(changeRequest.status) }}
              </span>
            </div>
            <div
              class="text-caption q-mb-xs"
              style="color: var(--text-muted)"
              v-if="changeRequest && changeRequest.status === 'declined'"
            >
              {{ $t('changeRequests.form.declineReasonLabel') }}
              <span style="color: var(--text-color)">
                {{ $t(`changeRequests.decline_reason.${changeRequest.decline_reason}`) }}
              </span>
            </div>
            <div class="text-caption" style="color: var(--text-muted)" v-if="changeRequest">
              {{
                t('changeRequests.detail.meta', {
                  id: changeRequest.id,
                  created: formatDate(changeRequest.created_at),
                  updated: formatDate(changeRequest.updated_at),
                })
              }}
            </div>
          </q-card-section>
        </q-card>

        <ScopeChangePanel
          ref="scopePanel"
          v-if="changeRequest"
          :key="changeRequest.id"
          :id="changeRequest.id"
          :version="changeRequest.version"
          @ready="scopeCanImplement = !!$event && !$event.current.legacy"
        >
          <template v-if="canSubmit || canAssess || canMarkImplemented" #actions>
            <q-btn
              v-if="canSubmit"
              unelevated
              color="primary"
              icon="send"
              :label="t('changeRequests.detail.action.submit')"
              @click="submitChangeRequest"
            />
            <q-btn
              v-if="canAssess"
              unelevated
              color="primary"
              icon="analytics"
              :label="t('changeRequests.detail.action.assess')"
              :disable="isAssessing"
              @click="openAssessDialog"
            />
            <q-btn
              v-if="canMarkImplemented"
              unelevated
              color="positive"
              icon="task_alt"
              :label="t('changeRequests.detail.action.markImplemented')"
              @click="markImplemented"
            />
          </template>
        </ScopeChangePanel>

        <ScopeSection
          class="q-mb-md"
          :title="t('changeRequests.detail.specsTitle')"
          icon="description"
        >
          <div class="row q-col-gutter-md detail-meta-grid" v-if="changeRequest">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.project') }}
              </div>
              <div style="color: var(--text-color)">
                <router-link :to="`/projects/${changeRequest.project_id}`">{{
                  changeRequest.project_name || t('common.project')
                }}</router-link>
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('columns.changeRequest.client') }}
              </div>
              <div style="color: var(--text-color)">
                <router-link :to="`/clients/${changeRequest.client_id}`">{{
                  changeRequest.client_name || t('columns.changeRequest.client')
                }}</router-link>
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('columns.changeRequest.origin') }}
              </div>
              <div style="color: var(--text-color)">
                {{ originLabel(changeRequest.origin) }}
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.taskLabel') }}
              </div>
              <div style="color: var(--text-color)">
                <router-link v-if="changeRequest.task_id" :to="`/tasks/${changeRequest.task_id}`">{{
                  t('changeRequests.form.taskLabel')
                }}</router-link
                ><span v-else>{{ t('common.empty') }}</span>
              </div>
            </div>
          </div>
        </ScopeSection>
      </div>

      <div class="col-12 col-md-2" style="position: sticky; top: 80px">
        <q-card
          flat
          bordered
          style="background-color: var(--card-bg); border-color: var(--border-color)"
        >
          <q-list separator>
            <q-item
              v-if="changeRequest && changeRequest.status !== 'draft'"
              clickable
              v-ripple
              class="q-pa-md"
              @click="scopePanel?.focusConversation()"
              ><q-item-section avatar><q-icon name="forum" color="primary" /></q-item-section
              ><q-item-section
                >{{ t('scope.conversation') }}
                <ConversationUnreadBadge
                  v-if="changeRequest"
                  :project-id="changeRequest.project_id"
                  class="q-ml-sm" /></q-item-section
            ></q-item>
            <q-item clickable v-ripple @click="goBack" class="q-pa-md">
              <q-item-section avatar>
                <q-icon name="arrow_back" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('changeRequests.detail.action.back') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="canEdit"
              clickable
              v-ripple
              class="text-primary q-pa-md"
              @click="goToEdit"
            >
              <q-item-section avatar>
                <q-icon name="edit" color="primary" />
              </q-item-section>
              <q-item-section>{{ t('changeRequests.detail.action.edit') }}</q-item-section>
            </q-item>

            <q-item
              v-if="canSubmit"
              clickable
              v-ripple
              class="q-pa-md"
              @click="submitChangeRequest"
            >
              <q-item-section avatar>
                <q-icon name="send" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('changeRequests.detail.action.submit') }}
              </q-item-section>
            </q-item>

            <q-item v-if="canAssess" clickable v-ripple class="q-pa-md" @click="openAssessDialog">
              <q-item-section avatar>
                <q-icon name="analytics" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('changeRequests.detail.action.assess') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="canMarkImplemented"
              clickable
              v-ripple
              class="q-pa-md"
              @click="markImplemented"
            >
              <q-item-section avatar>
                <q-icon name="task_alt" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('changeRequests.detail.action.markImplemented') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="canCancel"
              clickable
              v-ripple
              class="q-pa-md"
              @click="cancelChangeRequest"
            >
              <q-item-section avatar>
                <q-icon name="cancel" style="color: var(--text-color)" />
              </q-item-section>
              <q-item-section style="color: var(--text-color)">
                {{ t('changeRequests.detail.action.cancel') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="canDelete"
              clickable
              v-ripple
              class="text-negative q-pa-md"
              @click="removeChangeRequest"
            >
              <q-item-section avatar>
                <q-icon name="delete" color="negative" />
              </q-item-section>
              <q-item-section>{{ t('changeRequests.detail.action.delete') }}</q-item-section>
            </q-item>
          </q-list>
        </q-card>
      </div>

      <q-dialog v-model="showAssessDialog" persistent>
        <q-card style="min-width: min(92vw, 640px)">
          <q-card-section>
            <div class="text-h6">{{ t('changeRequests.detail.assessDialogTitle') }}</div>
          </q-card-section>

          <q-card-section class="q-pt-none row q-col-gutter-md">
            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="assessForm.impact_hours"
                :label="t('changeRequests.form.impactHoursLabel')"
                :decimal-places="2"
                :error="hasAssessHoursError"
              />
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="assessForm.impact_cost"
                :label="t('changeRequests.form.impactCostLabel')"
                :decimal-places="4"
                :error="hasAssessCostError"
              />
            </div>

            <div class="col-12 col-md-4">
              <NumericTextInput
                v-model="assessForm.schedule_impact_days"
                :label="t('changeRequests.form.scheduleImpactDaysLabel')"
                :only-integers="true"
                :error="hasAssessDaysError"
              />
            </div>

            <div class="col-12 q-mt-sm">
              <div class="text-subtitle2 q-mb-sm" style="color: var(--text-color)">
                {{ t('changeRequests.detail.assessDialogAssessmentLabel') }}
              </div>

              <p class="text-muted">{{ t('scope.fixedHint') }}</p>
              <q-select
                v-model="targetMilestone"
                :options="milestoneOptions"
                emit-value
                map-options
                clearable
                :label="t('scope.milestone')"
                :hint="t('scope.milestoneHint')"
                class="q-mb-md"
              />
              <MarkdownRenderer
                v-model="assessForm.technical_assessment"
                :edit-mode="true"
                :enable-preview-toggle="true"
              />

              <div v-if="hasAssessDescriptionError" class="text-negative text-caption q-mt-xs">
                {{ t('changeRequests.detail.assessDialogAssessmentLabel') }}
              </div>
            </div>
          </q-card-section>

          <q-card-actions align="right">
            <q-btn flat :label="t('common.cancel')" @click="showAssessDialog = false" />
            <q-btn
              color="primary"
              unelevated
              :label="t('changeRequests.detail.assessDialogSave')"
              :loading="isAssessing"
              @click="saveAssessment"
            />
          </q-card-actions>
        </q-card>
      </q-dialog>
    </template>
  </q-page>
</template>

<script setup lang="ts">
import ConversationUnreadBadge from 'src/components/ConversationUnreadBadge.vue';
import ScopeSection from 'src/components/ScopeSection.vue';
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
const contentLoading = ref(true);
import ScopeChangePanel from 'src/components/ScopeChangePanel.vue';
const scopePanel = ref<InstanceType<typeof ScopeChangePanel> | null>(null);
import { computed, onMounted, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { ScopeService } from 'src/services/scope';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { useUI } from 'src/composables/useUI';
import { ChangeRequestsService } from 'src/services/changeRequests';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { normalizeId } from 'src/utils/normalizeId';
import { normalizeNumericText, parseNumericText } from 'src/utils/numericInput';
import type {
  ChangeRequestDetailDto,
  ChangeRequestOrigin,
  ChangeRequestStatus,
} from 'src/types/changeRequest';

interface AssessFormState {
  technical_assessment: string;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: string;
}

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, confirmAction, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const changeRequest = ref<ChangeRequestDetailDto | null>(null);
const changeRequestId = String(route.params.id || '');

const scopeCanImplement = ref(false);
const showAssessDialog = ref(false);
const targetMilestone = ref<string | null>(null);
const milestoneOptions = ref<{ label: string; value: string }[]>([]);
const isAssessing = ref(false);
const hasAttemptedAssess = ref(false);

const assessForm = reactive<AssessFormState>({
  technical_assessment: '',
  impact_hours: '0',
  impact_cost: '0',
  schedule_impact_days: '0',
});

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback,
  });

const statusLabel = (status: ChangeRequestStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`changeRequests.status.${status}`);
};

const originLabel = (origin: ChangeRequestOrigin | null | undefined) => {
  if (!origin) return t('common.empty');
  return t(`changeRequests.origin.${origin}`);
};

const canEdit = computed(() => {
  if (!changeRequest.value) return false;
  if (changeRequest.value.origin === 'client_portal') return false;

  return !['approved', 'declined', 'implemented', 'cancelled'].includes(changeRequest.value.status);
});

const canSubmit = computed(() => changeRequest.value?.status === 'draft');
const canAssess = computed(() =>
  ['submitted', 'assessed'].includes(changeRequest.value?.status || ''),
);
const canMarkImplemented = computed(
  () => changeRequest.value?.status === 'approved' && scopeCanImplement.value,
);
const canCancel = computed(() => {
  if (!changeRequest.value) return false;
  if (changeRequest.value.origin !== 'freelancer_dashboard') return false;

  return !['approved', 'declined', 'implemented', 'cancelled'].includes(changeRequest.value.status);
});
const canDelete = computed(() => false);

const hasAssessDescriptionError = computed(
  () => hasAttemptedAssess.value && assessForm.technical_assessment.trim().length === 0,
);
const hasAssessHoursError = computed(() => {
  if (!hasAttemptedAssess.value) return false;
  const value = parseNumericText(assessForm.impact_hours, { decimalPlaces: 2 });
  return Number.isNaN(value) || value < 0;
});
const hasAssessCostError = computed(() => {
  if (!hasAttemptedAssess.value) return false;
  const value = parseNumericText(assessForm.impact_cost, { decimalPlaces: 4 });
  return Number.isNaN(value) || value < 0;
});
const hasAssessDaysError = computed(() => {
  if (!hasAttemptedAssess.value) return false;
  const value = parseNumericText(assessForm.schedule_impact_days, { onlyIntegers: true });
  return Number.isNaN(value) || value < 0;
});

const loadChangeRequest = async () => {
  contentLoading.value = true;

  try {
    changeRequest.value = await ChangeRequestsService.getById(changeRequestId);
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.loadError'), err);
    void router.push('/change-requests');
  } finally {
    contentLoading.value = false;
  }
};

const goBack = () => {
  void router.push('/change-requests');
};

const goToEdit = () => {
  if (!changeRequest.value) return;
  void router.push(`/change-requests/${changeRequest.value.id}/edit`);
};

const submitChangeRequest = async () => {
  if (!changeRequest.value) return;

  showLoading(t('changeRequests.detail.submitting'));
  try {
    changeRequest.value = await ChangeRequestsService.submit(
      changeRequest.value.id,
      changeRequest.value.version,
    );
    notify(t('changeRequests.detail.submitSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.submitError'), err);
  } finally {
    hideLoading();
  }
};

const openAssessDialog = () => {
  if (!changeRequest.value) return;

  assessForm.technical_assessment = changeRequest.value.technical_assessment || '';
  assessForm.impact_hours = normalizeNumericText(String(changeRequest.value.impact_hours ?? '0'), {
    decimalPlaces: 2,
  });
  assessForm.impact_cost = normalizeNumericText(String(changeRequest.value.impact_cost ?? '0'), {
    decimalPlaces: 4,
  });
  assessForm.schedule_impact_days = normalizeNumericText(
    String(changeRequest.value.schedule_impact_days ?? 0),
    {
      onlyIntegers: true,
    },
  );
  hasAttemptedAssess.value = false;
  targetMilestone.value = null;
  milestoneOptions.value = [];
  void ScopeService.project(changeRequest.value.project_id)
    .then((p) => {
      milestoneOptions.value = p.milestones
        .filter((m) => m.due_date && ['open', 'review'].includes(m.status))
        .map((m) => ({ label: `${m.name} — ${m.due_date}`, value: m.id }));
    })
    .catch((e) => notifyError(t('scope.failure'), e));
  showAssessDialog.value = true;
};

const saveAssessment = async () => {
  if (!changeRequest.value) return;

  hasAttemptedAssess.value = true;
  if (
    hasAssessDescriptionError.value ||
    hasAssessHoursError.value ||
    hasAssessCostError.value ||
    hasAssessDaysError.value
  ) {
    return;
  }

  isAssessing.value = true;
  showLoading(t('changeRequests.detail.assessing'));

  try {
    changeRequest.value = await ChangeRequestsService.assess(changeRequest.value.id, {
      version: changeRequest.value.version,
      target_milestone_id: targetMilestone.value,
      technical_assessment: assessForm.technical_assessment.trim(),
      impact_hours: String(parseNumericText(assessForm.impact_hours, { decimalPlaces: 2 })),
      impact_cost: String(parseNumericText(assessForm.impact_cost, { decimalPlaces: 4 })),
      schedule_impact_days: parseNumericText(assessForm.schedule_impact_days, {
        onlyIntegers: true,
      }),
    });

    showAssessDialog.value = false;
    notify(t('changeRequests.detail.assessSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.assessError'), err);
  } finally {
    isAssessing.value = false;
    hideLoading();
  }
};

const markImplemented = async () => {
  if (!changeRequest.value) return;

  showLoading(t('changeRequests.detail.action.markImplemented'));
  try {
    changeRequest.value = await ChangeRequestsService.markImplemented(
      changeRequest.value.id,
      changeRequest.value.version,
    );
    notify(t('changeRequests.detail.implementSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.implementError'), err);
  } finally {
    hideLoading();
  }
};

const cancelChangeRequest = async () => {
  if (!changeRequest.value) return;

  const confirmed = await confirmAction({
    title: t('changeRequests.detail.cancelConfirmTitle'),
    message: t('changeRequests.detail.cancelConfirmMessage'),
    okLabel: t('changeRequests.detail.action.cancel'),
    okColor: 'warning',
  });
  if (!confirmed) return;

  showLoading(t('changeRequests.detail.action.cancel'));
  try {
    changeRequest.value = await ChangeRequestsService.cancel(
      changeRequest.value.id,
      changeRequest.value.version,
    );
    notify(t('changeRequests.detail.cancelSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.cancelError'), err);
  } finally {
    hideLoading();
  }
};

const removeChangeRequest = async () => {
  if (!changeRequest.value) return;

  const confirmed = await confirmDelete(t('changeRequests.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('changeRequests.detail.deleting'));
  try {
    await ChangeRequestsService.delete(changeRequest.value.id);
    notify(t('changeRequests.detail.deleteSuccess'), 'positive');
    void router.push('/change-requests');
  } catch (err: unknown) {
    notifyError(t('changeRequests.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

onMounted(() => {
  void loadChangeRequest();
});
</script>
