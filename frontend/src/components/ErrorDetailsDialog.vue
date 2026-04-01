<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide" persistent>
    <q-card class="q-dialog-plugin custom-error-dialog">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6 text-negative row items-center q-gutter-sm">
          <q-icon name="error_outline" size="sm" />
          <span>Detalles del Error (500)</span>
        </div>
        <q-space />
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-card-section class="q-mt-sm">
        <div class="text-subtitle2 q-mb-sm text-weight-medium">
          El servidor encontró un error interno. Payload técnico:
        </div>
        <q-scroll-area style="height: 250px" class="error-container rounded-borders">
          <pre class="q-px-md q-py-sm">{{ formattedPayload }}</pre>
        </q-scroll-area>
      </q-card-section>

      <q-card-actions align="right" class="q-px-md q-pb-md">
        <q-btn flat label="Cerrar" style="color: var(--text-muted)" @click="onDialogOK" />
        <q-btn
          unelevated
          icon="bug_report"
          label="Reportar Error"
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

const props = defineProps<{
  payload: unknown;
}>();

defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();
const $q = useQuasar();

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
    message: 'Reporte interno de error enviado correctamente (TODO)',
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
