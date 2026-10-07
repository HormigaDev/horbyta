<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide" persistent>
    <q-card class="q-dialog-plugin custom-error-dialog">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6 text-negative row items-center q-gutter-sm">
          <q-icon name="error_outline" size="sm" />
          <span>{{ dialogTitle }}</span>
        </div>
        <q-space />
        <q-btn icon="close" flat round dense :aria-label="t('common.close')" v-close-popup />
      </q-card-section>

      <q-card-section class="q-mt-sm">
        <div class="text-subtitle2 q-mb-sm text-weight-medium">
          {{ dialogSubtitle }}
        </div>
        <q-scroll-area style="height: 250px" class="error-container rounded-borders">
          <pre class="q-px-md q-py-sm">{{ formattedPayload }}</pre>
        </q-scroll-area>
      </q-card-section>

      <q-card-actions align="right" class="q-px-md q-pb-md">
        <q-btn
          flat
          :label="t('common.close')"
          style="color: var(--text-muted)"
          @click="onDialogOK"
        />
        <q-btn
          unelevated
          icon="bug_report"
          :label="t('error.dialog.reportButton')"
          color="negative"
          @click="onReport"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useDialogPluginComponent, useQuasar } from 'quasar';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  payload: unknown;
}>();

defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();
const $q = useQuasar();
const { t } = useI18n();

const toPayloadRecord = (value: unknown): Record<string, unknown> | null => {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return null;
};

const statusCode = computed<number | null>(() => {
  const payloadRecord = toPayloadRecord(props.payload);
  if (!payloadRecord) {
    return null;
  }

  const rawStatus = payloadRecord.status;
  if (typeof rawStatus === 'number' && Number.isFinite(rawStatus)) {
    return Math.trunc(rawStatus);
  }

  if (typeof rawStatus === 'string') {
    const parsed = Number.parseInt(rawStatus, 10);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }

  return null;
});

const dialogTitle = computed(() => {
  if (statusCode.value === null) {
    return t('error.dialog.title');
  }

  return t('error.dialog.titleWithStatus', { status: statusCode.value });
});

const dialogSubtitle = computed(() => {
  if (statusCode.value === null) {
    return t('error.dialog.subtitle');
  }

  if (statusCode.value >= 500) {
    return t('error.dialog.serverSubtitle');
  }

  if (statusCode.value >= 400) {
    return t('error.dialog.clientSubtitle');
  }

  return t('error.dialog.subtitle');
});

const formattedPayload = computed(() => {
  try {
    return JSON.stringify(props.payload, null, 2);
  } catch (_e) {
    return String(props.payload);
  }
});

const onReport = () => {
  // TODO: Integrar lógica de reporte de errores al servidor o sistema Sentry/Datadog
  $q.notify({
    type: 'positive',
    message: t('error.dialog.reportSuccess'),
    position: 'top-right',
  });
  onDialogOK();
};
</script>

<style lang="scss" scoped>
.custom-error-dialog {
  min-width: 450px;
  max-width: 90vw;
  background-color: var(--card-bg) !important;
  color: var(--text-color) !important;

  .error-container {
    background-color: var(--q-dark-page) !important;
    border: 1px solid var(--border-color);

    pre {
      font-family: monospace;
      color: var(--text-color);
      white-space: pre-wrap;
      word-break: break-all;
      margin: 0;
    }
  }
}
</style>
