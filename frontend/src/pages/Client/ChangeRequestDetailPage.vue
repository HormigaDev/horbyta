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
            <h1 class="text-h5 text-weight-bold q-ma-none" style="color: var(--text-color)">
              {{ changeRequest?.title || t('common.loading') }}
            </h1>
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
            <!-- Rechazo con su motivo y, si existe, la explicación del cliente. -->
            <div
              v-if="
                changeRequest && changeRequest.status === 'declined' && changeRequest.decline_reason
              "
              class="decline-notice q-mt-md"
              role="note"
            >
              <q-icon name="block" size="20px" class="decline-notice__icon" aria-hidden="true" />
              <div>
                <span class="decline-notice__label">{{
                  t('changeRequests.form.declineReasonLabel')
                }}</span>
                <strong>{{
                  t(`changeRequests.decline_reason.${changeRequest.decline_reason}`)
                }}</strong>
                <blockquote
                  v-if="changeRequest.decline_comment"
                  class="decline-notice__comment"
                  v-text="changeRequest.decline_comment"
                />
              </div>
            </div>
          </q-card-section>
        </q-card>

        <ScopeChangePanel
          ref="scopePanel"
          v-if="changeRequest"
          :key="changeRequest.id"
          :id="changeRequest.id"
          :version="changeRequest.version"
          client
          @stale="loadChangeRequest"
          @ready="
            agreementReady =
              !!$event && $event.current.version === changeRequest.version && !$event.current.legacy
          "
        >
          <template v-if="canApproveOrDecline" #actions>
            <q-btn
              unelevated
              color="positive"
              icon="thumb_up"
              :label="t('client.changeRequests.detail.action.approve')"
              :disable="deciding"
              @click="approveChangeRequest"
            />
            <q-btn
              outline
              color="negative"
              icon="thumb_down"
              :label="t('client.changeRequests.detail.action.decline')"
              :disable="deciding"
              @click="showDeclineDialog = true"
            />
          </template>
        </ScopeChangePanel>

        <ScopeSection
          class="q-mb-md"
          :title="t('client.changeRequests.detail.specsTitle')"
          icon="description"
        >
          <div class="row q-col-gutter-md detail-meta-grid" v-if="changeRequest">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.project') }}
              </div>
              <div style="color: var(--text-color)">
                <router-link :to="`/client/projects/${changeRequest.project_id}`">{{
                  changeRequest.project_name || t('common.project')
                }}</router-link>
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
                {{ t('client.changeRequests.detail.action.back') }}
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
              <q-item-section>{{ t('client.changeRequests.detail.action.edit') }}</q-item-section>
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
                {{ t('client.changeRequests.detail.action.cancel') }}
              </q-item-section>
            </q-item>

            <q-item
              v-if="canApproveOrDecline"
              clickable
              v-ripple
              class="q-pa-md"
              :disable="deciding"
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
        <q-card flat bordered style="width: 480px; max-width: 92vw">
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
            <!-- "Otro" exige que el cliente explique el motivo con sus palabras. -->
            <q-input
              v-if="declineReason === 'other'"
              v-model="declineComment"
              outlined
              autofocus
              type="textarea"
              autogrow
              maxlength="2000"
              counter
              class="q-mt-md"
              :label="t('changeRequests.detail.declineCommentLabel')"
              :rules="[(v) => !!String(v ?? '').trim() || t('scope.explanationRequired')]"
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
              :disable="
                !declineReason || (declineReason === 'other' && !declineComment.trim()) || deciding
              "
              @click="declineChangeRequest"
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
import { isApiHttpError } from 'src/boot/fetch';
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { ClientChangeRequestsService } from 'src/services/clientChangeRequests';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ChangeRequestStatus, ClientChangeRequestDetailDto } from 'src/types/changeRequest';
import { getChangeRequestsDeclineReasons } from 'src/data/constants';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmAction, showLoading, hideLoading } = useUI();
const userPreferencesStore = useDisplayPreferences();

const changeRequest = ref<ClientChangeRequestDetailDto | null>(null);
const showDeclineDialog = ref(false);
const declineReason = ref('');
const declineComment = ref('');
const agreementReady = ref(false);
const deciding = ref(false);
const changeRequestId = String(route.params.id || '');

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

const canApproveOrDecline = computed(
  () => changeRequest.value?.status === 'assessed' && agreementReady.value,
);

const loadChangeRequest = async () => {
  contentLoading.value = true;
  try {
    changeRequest.value = await ClientChangeRequestsService.getById(changeRequestId);
  } catch (err: unknown) {
    notifyError(t('client.changeRequests.detail.loadError'), err);
    void router.push('/client/change-requests');
  } finally {
    deciding.value = false;
    contentLoading.value = false;
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
    deciding.value = false;
    hideLoading();
  }
};

const approveChangeRequest = async () => {
  if (!changeRequest.value || !agreementReady.value || deciding.value) return;
  deciding.value = true;
  showLoading(t('client.changeRequests.detail.action.approve'));
  try {
    changeRequest.value = await ClientChangeRequestsService.approve(changeRequest.value.id, {
      version: changeRequest.value.version,
    });
    notify(t('client.changeRequests.detail.approveSuccess'), 'positive');
  } catch (err: unknown) {
    // El motivo real del backend; un conflicto además recarga la oferta vigente.
    notifyError(t('client.changeRequests.detail.approveError'), err);
    if (isApiHttpError(err) && err.status === 409) await loadChangeRequest();
  } finally {
    deciding.value = false;
    hideLoading();
  }
};

const declineChangeRequest = async () => {
  if (!changeRequest.value) return;
  if (!declineReason.value || !agreementReady.value || deciding.value) return;
  if (declineReason.value === 'other' && !declineComment.value.trim()) return;
  deciding.value = true;

  showLoading(t('client.changeRequests.detail.action.decline'));
  try {
    changeRequest.value = await ClientChangeRequestsService.decline(
      changeRequest.value.id,
      declineReason.value,
      changeRequest.value.version,
      declineReason.value === 'other' ? declineComment.value.trim() : null,
    );
    notify(t('client.changeRequests.detail.declineSuccess'), 'positive');
  } catch (err: unknown) {
    // El motivo real del backend; un conflicto además recarga la oferta vigente.
    notifyError(t('client.changeRequests.detail.declineError'), err);
    if (isApiHttpError(err) && err.status === 409) await loadChangeRequest();
  } finally {
    showDeclineDialog.value = false;
    deciding.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadChangeRequest();
});
</script>
