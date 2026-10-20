<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide">
    <q-card class="q-dialog-plugin error-details-dialog">
      <q-card-section class="row items-center no-wrap q-pb-sm">
        <q-icon name="data_object" size="sm" class="q-mr-sm error-details-dialog__icon" />
        <div class="text-h6 col">{{ dialogTitle }}</div>
        <q-btn icon="close" flat round dense :aria-label="t('common.close')" v-close-popup />
      </q-card-section>

      <q-card-section class="q-pt-none">
        <div v-if="summary.length" class="error-details-dialog__summary">
          <span v-for="item in summary" :key="item.label" class="error-details-dialog__chip">
            <span class="error-details-dialog__chip-label">{{ item.label }}</span>
            <span class="error-details-dialog__chip-value">{{ item.value }}</span>
          </span>
        </div>

        <div class="row items-center justify-between q-mt-md q-mb-sm">
          <div class="text-caption error-details-dialog__muted">{{ dialogSubtitle }}</div>
          <q-toggle
            v-model="showJson"
            dense
            icon="code"
            :label="t('error.details.showJson')"
            class="error-details-dialog__toggle"
          />
        </div>

        <div v-if="showJson" class="error-details-dialog__json">
          <!-- eslint-disable-next-line vue/no-v-html -- HTML escapado en highlightJson -->
          <pre v-html="highlightedJson" />
          <q-btn
            flat
            round
            dense
            size="sm"
            icon="content_copy"
            class="error-details-dialog__copy"
            :aria-label="t('error.details.copy')"
            @click="copyJson"
          />
        </div>
        <q-list v-else bordered separator class="error-details-dialog__tree">
          <template v-if="entries">
            <ErrorDetailsNode
              v-for="entry in entries"
              :key="entry.key"
              :name="entry.key"
              :value="entry.value"
            />
          </template>
          <ErrorDetailsNode v-else name="value" :value="payload" />
        </q-list>
      </q-card-section>

      <q-card-actions align="right" class="q-px-md q-pb-md">
        <q-btn
          flat
          no-caps
          icon="content_copy"
          :label="t('error.details.copyJson')"
          @click="copyJson"
        />
        <q-space />
        <q-btn
          flat
          :label="t('common.close')"
          class="error-details-dialog__muted"
          @click="onDialogOK"
        />
        <q-btn
          unelevated
          icon="bug_report"
          color="negative"
          :label="t('error.dialog.reportButton')"
          @click="onReport"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { copyToClipboard, useDialogPluginComponent, useQuasar } from 'quasar';
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import ErrorDetailsNode from './ErrorDetailsNode.vue';

const props = defineProps<{
  payload: unknown;
}>();

defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();
const $q = useQuasar();
const { t } = useI18n();
const showJson = ref(false);

const record = computed<Record<string, unknown> | null>(() => {
  const value = props.payload;
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
});

// En la vista amigable se omiten los campos repetidos: `message` y `error` son el mismo texto.
const entries = computed(() => {
  if (!record.value) return null;
  return Object.entries(record.value)
    .filter(([key, value]) => !(key === 'message' && value === record.value?.error))
    .map(([key, value]) => ({ key, value }));
});

const statusCode = computed<number | null>(() => {
  const raw = record.value?.status;
  const parsed =
    typeof raw === 'number' ? raw : typeof raw === 'string' ? Number.parseInt(raw, 10) : NaN;
  return Number.isFinite(parsed) ? Math.trunc(parsed) : null;
});

const dialogTitle = computed(() =>
  statusCode.value === null
    ? t('error.dialog.title')
    : t('error.dialog.titleWithStatus', { status: statusCode.value }),
);

const dialogSubtitle = computed(() => {
  if (statusCode.value !== null && statusCode.value >= 500) return t('error.details.serverHint');
  return t('error.details.hint');
});

const summary = computed(() => {
  const value = record.value;
  if (!value) return [];
  const items: { label: string; value: string }[] = [];
  if (typeof value.method === 'string' && typeof value.url === 'string') {
    items.push({ label: t('error.details.request'), value: `${value.method} ${value.url}` });
  }
  if (typeof value.code === 'string')
    items.push({ label: t('error.details.key.code'), value: value.code });
  return items;
});

const json = computed(() => {
  try {
    return JSON.stringify(props.payload, null, 2) ?? String(props.payload);
  } catch {
    return String(props.payload);
  }
});

const escapeHtml = (value: string) =>
  value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// Colorea claves, textos, números y literales del JSON ya formateado.
const highlightedJson = computed(() =>
  escapeHtml(json.value).replace(
    /("(?:\\.|[^"\\])*")(\s*:)?|\b(true|false|null)\b|(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g,
    (match, str: string | undefined, colon: string | undefined, literal: string | undefined) => {
      if (str)
        return colon
          ? `<span class="json-key">${str}</span>${colon}`
          : `<span class="json-string">${str}</span>`;
      if (literal) return `<span class="json-literal">${literal}</span>`;
      return `<span class="json-number">${match}</span>`;
    },
  ),
);

const onReport = () => {
  // TODO: Integrar lógica de reporte de errores al servidor o sistema Sentry/Datadog
  $q.notify({ type: 'positive', message: t('error.dialog.reportSuccess'), position: 'top-right' });
  onDialogOK();
};

const copyJson = async () => {
  try {
    await copyToClipboard(json.value);
    $q.notify({
      message: t('error.details.copied'),
      type: 'positive',
      position: 'top-right',
      timeout: 1500,
    });
  } catch {
    $q.notify({ message: t('error.details.copyFailed'), type: 'warning', position: 'top-right' });
  }
};
</script>

<style scoped>
.error-details-dialog {
  width: min(720px, 96vw);
  max-width: 96vw;
  border: 1px solid var(--border-color);
  border-radius: 14px;
  background: var(--card-bg);
  color: var(--text-color);
}

.error-details-dialog__icon,
.error-details-dialog__muted {
  color: var(--text-muted);
}

.error-details-dialog__summary {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.error-details-dialog__chip {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  overflow: hidden;
  font-size: 12px;
}

.error-details-dialog__chip-label {
  padding: 3px 8px;
  background: var(--bg-hover);
  color: var(--text-muted);
  white-space: nowrap;
}

.error-details-dialog__chip-value {
  padding: 3px 10px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.error-details-dialog__tree {
  border-radius: 10px;
  max-height: min(56vh, 460px);
  overflow-x: hidden;
  overflow-y: auto;
}

.error-details-dialog__json {
  position: relative;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--surface-page);
}

.error-details-dialog__json pre {
  margin: 0;
  padding: 14px 44px 14px 16px;
  max-height: min(56vh, 460px);
  overflow: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.6;
  white-space: pre;
  tab-size: 2;
}

.error-details-dialog__json :deep(.json-key) {
  color: var(--info-text);
}
.error-details-dialog__json :deep(.json-string) {
  color: var(--success-text);
}
.error-details-dialog__json :deep(.json-number) {
  color: var(--warning-text);
}
.error-details-dialog__json :deep(.json-literal) {
  color: var(--error-text);
}

.error-details-dialog__copy {
  position: absolute;
  top: 6px;
  right: 6px;
  color: var(--text-muted);
}
</style>
