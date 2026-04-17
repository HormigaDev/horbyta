<template>
  <q-dialog ref="dialogRef" @hide="onDialogHide" persistent>
    <q-card class="q-dialog-plugin bad-request-dialog">
      <q-card-section class="row items-center q-pb-none q-gutter-sm">
        <q-avatar color="warning" text-color="black" icon="rule" />
        <div class="text-h6 text-weight-bold">
          {{ t('error.badRequestDialog.title') }}
        </div>
        <q-space />
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-card-section>
        <div class="text-subtitle2 q-mb-md">
          {{ summary || t('error.badRequestDialog.subtitle') }}
        </div>

        <q-list bordered separator class="rounded-borders validation-list">
          <q-item v-for="(item, index) in orderedMessages" :key="`validation-${index}`">
            <q-item-section side>
              <q-badge rounded color="negative" text-color="white" :label="String(index + 1)" />
            </q-item-section>
            <q-item-section>
              {{ item }}
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>

      <q-card-actions align="right" class="q-px-md q-pb-md">
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
import { useDialogPluginComponent } from 'quasar';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  summary: string;
  messages: string[];
}>();

defineEmits([...useDialogPluginComponent.emits]);
const { dialogRef, onDialogHide, onDialogOK } = useDialogPluginComponent();
const { t } = useI18n();

const orderedMessages = computed(() => {
  const unique = Array.from(
    new Set(props.messages.map((item) => item.trim()).filter((item) => item)),
  );
  return unique.sort((left, right) => left.localeCompare(right, 'es'));
});
</script>

<style scoped>
.bad-request-dialog {
  min-width: 520px;
  max-width: 92vw;
  border: 1px solid var(--border-color);
  border-radius: 14px;
  background: var(--card-bg);
  color: var(--text-color);
}

.validation-list {
  max-height: 300px;
  overflow: auto;
}

@media (max-width: 640px) {
  .bad-request-dialog {
    min-width: 0;
    width: 96vw;
  }
}
</style>
