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
        <q-toggle
          v-model="form.group_notifications"
          color="primary"
          :label="t('settings.notifications.item.groupNotifications')"
        />
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">
          {{ t('settings.notifications.group.clientPortal') }}
        </div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.client_portal.change_request_created"
              color="primary"
              :label="t('settings.notifications.item.changeRequestCreated')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.client_portal.change_request_updated"
              color="primary"
              :label="t('settings.notifications.item.changeRequestUpdated')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.client_portal.change_request_approved"
              color="primary"
              :label="t('settings.notifications.item.changeRequestApproved')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.client_portal.change_request_declined"
              color="primary"
              :label="t('settings.notifications.item.changeRequestDeclined')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.client_portal.change_request_cancelled"
              color="primary"
              :label="t('settings.notifications.item.changeRequestCancelled')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.billing') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.invoice_opened"
              color="primary"
              :label="t('settings.notifications.item.invoiceOpened')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.invoice_paid"
              color="primary"
              :label="t('settings.notifications.item.invoicePaid')"
            />
          </div>
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.billing.payment_received"
              color="primary"
              :label="t('settings.notifications.item.paymentReceived')"
            />
          </div>
        </div>
      </div>

      <div class="settings-block q-pa-md">
        <div class="text-subtitle2 q-mb-sm">{{ t('settings.notifications.group.work') }}</div>
        <div class="row q-col-gutter-sm">
          <div class="col-12 col-md-6">
            <q-toggle
              v-model="form.work.milestone_released"
              color="primary"
              :label="t('settings.notifications.item.milestoneReleased')"
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
    group_notifications: false,
    client_portal: {
      change_request_created: true,
      change_request_updated: true,
      change_request_approved: true,
      change_request_declined: true,
      change_request_cancelled: true,
    },
    billing: {
      invoice_opened: true,
      invoice_paid: true,
      payment_received: true,
    },
    work: {
      milestone_released: true,
    },
  };
}

function buildNotificationsPayload(): UpdateNotificationPreferencesDto {
  return {
    group_notifications: form.group_notifications,
    client_portal: {
      change_request_created: form.client_portal.change_request_created,
      change_request_updated: form.client_portal.change_request_updated,
      change_request_approved: form.client_portal.change_request_approved,
      change_request_declined: form.client_portal.change_request_declined,
      change_request_cancelled: form.client_portal.change_request_cancelled,
    },
    billing: {
      invoice_opened: form.billing.invoice_opened,
      invoice_paid: form.billing.invoice_paid,
      payment_received: form.billing.payment_received,
    },
    work: {
      milestone_released: form.work.milestone_released,
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
    const next = value ?? createDefaultNotifications();

    form.group_notifications = next.group_notifications;
    Object.assign(form.client_portal, next.client_portal);
    Object.assign(form.billing, next.billing);
    Object.assign(form.work, next.work);
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
