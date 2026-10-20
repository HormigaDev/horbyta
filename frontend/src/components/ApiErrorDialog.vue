<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide" persistent>
    <q-card class="q-dialog-plugin api-error-dialog" role="alertdialog" :aria-label="title">
      <q-card-section class="row items-center no-wrap q-pb-none q-gutter-sm">
        <q-avatar :class="`api-error-dialog__icon api-error-dialog__icon--${tone}`" :icon="icon" />
        <div class="text-h6 text-weight-bold col">{{ title }}</div>
        <q-btn icon="close" flat round dense :aria-label="t('common.close')" v-close-popup />
      </q-card-section>

      <q-card-section>
        <p class="api-error-dialog__message">{{ message }}</p>
        <p v-if="hint" class="api-error-dialog__hint">{{ hint }}</p>
      </q-card-section>

      <q-card-actions align="right" class="q-px-md q-pb-md">
        <q-btn
          v-if="payload !== undefined"
          flat
          no-caps
          icon="data_object"
          class="api-error-dialog__details"
          :label="t('common.details')"
          @click="onShowPayload"
        />
        <q-btn
          unelevated
          color="primary"
          :label="t('error.badRequestDialog.accept')"
          @click="onDialogOK"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useDialogPluginComponent, useQuasar } from 'quasar';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import ErrorDetailsDialog from './ErrorDetailsDialog.vue';
import type { ApiErrorKind } from 'src/utils/apiErrors';

const props = defineProps<{
  kind: ApiErrorKind;
  message: string;
  hint?: string | null;
  payload?: unknown;
}>();

defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();
const { t } = useI18n();
const $q = useQuasar();

const ICONS: Record<ApiErrorKind, string> = {
  validation: 'rule',
  conflict: 'block',
  forbidden: 'lock',
  notFound: 'search_off',
  auth: 'login',
  server: 'error_outline',
};
const icon = computed(() => ICONS[props.kind]);
const tone = computed(() => (props.kind === 'server' ? 'negative' : 'warning'));
const title = computed(() => t(`error.apiDialog.title.${props.kind}`));

const onShowPayload = () => {
  $q.dialog({
    component: ErrorDetailsDialog,
    componentProps: { payload: props.payload },
  });
};
</script>

<style scoped>
.api-error-dialog {
  width: min(520px, 96vw);
  max-width: 96vw;
  border: 1px solid var(--border-color);
  border-radius: 14px;
  background: var(--card-bg);
  color: var(--text-color);
}

.api-error-dialog__icon {
  flex: 0 0 auto;
}

.api-error-dialog__icon--warning {
  background: var(--warning-bg);
  color: var(--warning-text);
}

.api-error-dialog__icon--negative {
  background: var(--error-bg);
  color: var(--error-text);
}

.api-error-dialog__message {
  margin: 0;
  font-size: 15px;
  line-height: 1.55;
}

.api-error-dialog__hint {
  margin: 10px 0 0;
  color: var(--text-muted);
  line-height: 1.5;
}

.api-error-dialog__details {
  color: var(--text-muted);
}
</style>
