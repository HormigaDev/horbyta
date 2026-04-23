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
            {{ changeRequest?.title || t('common.loading') }}
          </div>
          <div class="text-caption q-mt-xs" style="color: var(--text-muted)" v-if="changeRequest">
            {{ t('common.status') }}:
            <span style="color: var(--text-color)">
              {{ statusLabel(changeRequest.status) }}
            </span>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="changeRequest">
            {{
              t('client.changeRequests.detail.meta', {
                id: changeRequest.id,
                created: formatDate(changeRequest.created_at),
                updated: formatDate(changeRequest.updated_at),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('client.changeRequests.detail.specsTitle') }}
          </div>

          <div class="row q-col-gutter-md" v-if="changeRequest">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.project') }}
              </div>
              <div style="color: var(--text-color)">
                {{ changeRequest.project_name || t('common.empty') }}
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.taskLabel') }}
              </div>
              <div style="color: var(--text-color)">
                {{ changeRequest.task_id || t('common.empty') }}
              </div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.versionLabel', { version: 0 }) }}
              </div>
              <div style="color: var(--text-color)">{{ changeRequest.version }}</div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.impactHoursLabel') }}
              </div>
              <div style="color: var(--text-color)">{{ changeRequest.impact_hours }}</div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.impactCostLabel') }}
              </div>
              <div style="color: var(--text-color)">{{ changeRequest.impact_cost }}</div>
            </div>

            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('changeRequests.form.scheduleImpactDaysLabel') }}
              </div>
              <div style="color: var(--text-color)">{{ changeRequest.schedule_impact_days }}</div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('client.changeRequests.detail.clientRequestTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="
                changeRequest?.client_request_description ||
                t('client.changeRequests.detail.emptyClientRequest')
              "
            />
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
            {{ t('client.changeRequests.detail.technicalAssessmentTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="
                changeRequest?.technical_assessment ||
                t('client.changeRequests.detail.emptyTechnicalAssessment')
              "
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
              {{ t('client.changeRequests.detail.action.back') }}
            </q-item-section>
          </q-item>

          <q-item v-if="canEdit" clickable v-ripple class="text-primary q-pa-md" @click="goToEdit">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('client.changeRequests.detail.action.edit') }}</q-item-section>
          </q-item>

          <q-item v-if="canCancel" clickable v-ripple class="q-pa-md" @click="cancelChangeRequest">
            <q-item-section avatar>
              <q-icon name="cancel" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('client.changeRequests.detail.action.cancel') }}
            </q-item-section>
          </q-item>

          <q-item
            v-if="canApproveOrDecline"
            clickable
            v-ripple
            class="q-pa-md"
            @click="approveChangeRequest"
          >
            <q-item-section avatar>
              <q-icon name="thumb_up" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('client.changeRequests.detail.action.approve') }}
            </q-item-section>
          </q-item>

          <q-item
            v-if="canApproveOrDecline"
            clickable
            v-ripple
            class="q-pa-md"
            @click="showDeclineDialog = true"
          >
            <q-item-section avatar>
              <q-icon name="thumb_down" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ t('client.changeRequests.detail.action.decline') }}
            </q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
    <q-dialog v-model="showDeclineDialog" persistent>
      <q-card flat bordered style="width: 400px">
        <q-card-section>
          <span class="text-h6 text-weight-bold">
            {{ $t('client.changeRequests.detail.decline.title') }}
          </span>
        </q-card-section>
        <q-card-section>
          <q-select
            outlined
            v-model="declineReason"
            :options="getChangeRequestsDeclineReasons($t)"
            map-options
            emit-value
            :label="$t('changeRequests.detail.decline_reason')"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn
            color="secondary"
            :label="$t('common.cancel')"
            @click="showDeclineDialog = false"
          />
          <q-btn
            color="negative"
            :label="$t('client.changeRequests.detail.action.decline')"
            :disable="!declineReason"
            @click="declineChangeRequest"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { ClientChangeRequestsService } from 'src/services/clientChangeRequests';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ChangeRequestStatus, ClientChangeRequestDetailDto } from 'src/types/changeRequest';
import { getChangeRequestsDeclineReasons } from 'src/data/constants';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmAction, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const changeRequest = ref<ClientChangeRequestDetailDto | null>(null);
const showDeclineDialog = ref(false);
const declineReason = ref('');
const changeRequestId = String(route.params.id || '');

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const statusLabel = (status: ChangeRequestStatus | null | undefined) => {
  if (!status) return t('common.empty');
  return t(`changeRequests.status.${status}`);
};

const canEdit = computed(() => {
  if (!changeRequest.value) return false;
  return ['draft', 'submitted'].includes(changeRequest.value.status);
});

const canCancel = computed(() => {
  if (!changeRequest.value) return false;
  return (
    changeRequest.value.origin === 'client_portal' &&
    ['draft', 'submitted'].includes(changeRequest.value.status)
  );
});

const canApproveOrDecline = computed(() => changeRequest.value?.status === 'assessed');

const loadChangeRequest = async () => {
  showLoading(t('client.changeRequests.detail.loading'));
  try {
    changeRequest.value = await ClientChangeRequestsService.getById(changeRequestId);
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.detail.loadError'), err);
    void router.push('/client/change-requests');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/client/change-requests');
};

const goToEdit = () => {
  if (!changeRequest.value) return;
  void router.push(`/client/change-requests/${changeRequest.value.id}/edit`);
};

const cancelChangeRequest = async () => {
  if (!changeRequest.value) return;

  const confirmed = await confirmAction({
    title: t('client.changeRequests.detail.cancelConfirmTitle'),
    message: t('client.changeRequests.detail.cancelConfirmMessage'),
    okLabel: t('client.changeRequests.detail.action.cancel'),
    okColor: 'warning',
  });
  if (!confirmed) return;

  showLoading(t('client.changeRequests.detail.action.cancel'));
  try {
    changeRequest.value = await ClientChangeRequestsService.cancel(changeRequest.value.id, {
      version: changeRequest.value.version,
    });
    notify(t('client.changeRequests.detail.cancelSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.detail.cancelError'), err);
  } finally {
    hideLoading();
  }
};

const approveChangeRequest = async () => {
  if (!changeRequest.value) return;

  showLoading(t('client.changeRequests.detail.action.approve'));
  try {
    changeRequest.value = await ClientChangeRequestsService.approve(changeRequest.value.id, {});
    notify(t('client.changeRequests.detail.approveSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.detail.approveError'), err);
  } finally {
    hideLoading();
  }
};

const declineChangeRequest = async () => {
  if (!changeRequest.value) return;
  if (!declineReason.value) return;

  showLoading(t('client.changeRequests.detail.action.decline'));
  try {
    changeRequest.value = await ClientChangeRequestsService.decline(
      changeRequest.value.id,
      declineReason.value,
    );
    notify(t('client.changeRequests.detail.declineSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.detail.declineError'), err);
  } finally {
    showDeclineDialog.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadChangeRequest();
});
</script>
