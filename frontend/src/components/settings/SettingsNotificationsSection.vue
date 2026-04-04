<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ t('settings.notifications.title') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ t('settings.notifications.subtitle') }}
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="q-gutter-y-md">
      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">
          {{ t('settings.notifications.group.clientsProjects') }}
        </div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.clients_projects.proposal_decision"
              color="primary"
              :label="t('settings.notifications.item.proposalDecision')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.clients_projects.deliverable_approved"
              color="primary"
              :label="t('settings.notifications.item.deliverableApproved')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.clients_projects.comment_or_review"
              color="primary"
              :label="t('settings.notifications.item.commentOrReview')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.clients_projects.added_to_new_project"
              color="primary"
              :label="t('settings.notifications.item.addedToProject')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.deliveries') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.deliveries_deadlines.deadline_24h_48h"
              color="primary"
              :label="t('settings.notifications.item.deadline24h48h')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.deliveries_deadlines.deliverable_overdue"
              color="primary"
              :label="t('settings.notifications.item.deliverableOverdue')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.deliveries_deadlines.milestone_completed"
              color="primary"
              :label="t('settings.notifications.item.milestoneCompleted')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.contracts') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.contracts.contract_sent"
              color="primary"
              :label="t('settings.notifications.item.contractSent')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.contracts.contract_signed"
              color="primary"
              :label="t('settings.notifications.item.contractSigned')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.contracts.contract_expiring"
              color="primary"
              :label="t('settings.notifications.item.contractExpiring')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.billing') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.invoice_sent"
              color="primary"
              :label="t('settings.notifications.item.invoiceSent')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.invoice_overdue_unpaid"
              color="primary"
              :label="t('settings.notifications.item.invoiceOverdue')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.payment_received"
              color="primary"
              :label="t('settings.notifications.item.paymentReceived')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.auto_payment_reminder"
              color="primary"
              :label="t('settings.notifications.item.autoPaymentReminder')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.time') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.time_tracking.long_running_timer"
              color="primary"
              :label="t('settings.notifications.item.longRunningTimer')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.time_tracking.weekly_hours_summary"
              color="primary"
              :label="t('settings.notifications.item.weeklySummary')"
            />
          </div>
        </div>
      </div>
    </q-card-section>

    <q-card-actions align="right">
      <q-btn
        color="primary"
        unelevated
        :label="t('settings.notifications.saveButton')"
        :loading="isSaving"
        @click="saveNotifications"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import type {
  NotificationPreferences,
  UpdateNotificationPreferencesDto,
  UserSettings,
} from 'src/types/user';

const props = defineProps<{
  notifications: NotificationPreferences | null;
}>();

const emit = defineEmits<{
  updated: [value: UserSettings];
}>();

const { t } = useI18n();
const { notify, notifyError } = useUI();
const isSaving = ref(false);

const form = reactive<NotificationPreferences>(createDefaultNotifications());

function createDefaultNotifications(): NotificationPreferences {
  return {
    clients_projects: {
      proposal_decision: true,
      deliverable_approved: true,
      comment_or_review: true,
      added_to_new_project: true,
    },
    deliveries_deadlines: {
      deadline_24h_48h: true,
      deliverable_overdue: true,
      milestone_completed: true,
    },
    contracts: {
      contract_sent: true,
      contract_signed: true,
      contract_expiring: true,
    },
    billing: {
      invoice_sent: true,
      invoice_overdue_unpaid: true,
      payment_received: true,
      auto_payment_reminder: true,
    },
    time_tracking: {
      long_running_timer: true,
      weekly_hours_summary: true,
    },
  };
}

function buildNotificationsPayload(): UpdateNotificationPreferencesDto {
  return {
    clients_projects: {
      proposal_decision: form.clients_projects.proposal_decision,
      deliverable_approved: form.clients_projects.deliverable_approved,
      comment_or_review: form.clients_projects.comment_or_review,
      added_to_new_project: form.clients_projects.added_to_new_project,
    },
    deliveries_deadlines: {
      deadline_24h_48h: form.deliveries_deadlines.deadline_24h_48h,
      deliverable_overdue: form.deliveries_deadlines.deliverable_overdue,
      milestone_completed: form.deliveries_deadlines.milestone_completed,
    },
    contracts: {
      contract_sent: form.contracts.contract_sent,
      contract_signed: form.contracts.contract_signed,
      contract_expiring: form.contracts.contract_expiring,
    },
    billing: {
      invoice_sent: form.billing.invoice_sent,
      invoice_overdue_unpaid: form.billing.invoice_overdue_unpaid,
      payment_received: form.billing.payment_received,
      auto_payment_reminder: form.billing.auto_payment_reminder,
    },
    time_tracking: {
      long_running_timer: form.time_tracking.long_running_timer,
      weekly_hours_summary: form.time_tracking.weekly_hours_summary,
    },
  };
}

async function saveNotifications() {
  isSaving.value = true;

  try {
    const updated = await UsersService.updateNotifications(buildNotificationsPayload());
    emit('updated', updated);
    notify(t('settings.notifications.success'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.notifications.error'), error);
  } finally {
    isSaving.value = false;
  }
}

watch(
  () => props.notifications,
  (value) => {
    if (!value) return;

    Object.assign(form.clients_projects, value.clients_projects);
    Object.assign(form.deliveries_deadlines, value.deliveries_deadlines);
    Object.assign(form.contracts, value.contracts);
    Object.assign(form.billing, value.billing);
    Object.assign(form.time_tracking, value.time_tracking);
  },
  { immediate: true },
);
</script>

<style scoped>
.settings-block {
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--card-bg);
}
</style>
