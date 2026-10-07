<template>
  <q-dialog
    :model-value="modelValue"
    @update:model-value="(value) => emit('update:modelValue', value)"
  >
    <q-card class="filter-dialog">
      <q-card-section class="filter-dialog__head">
        <div class="filter-dialog__title">{{ title }}</div>
        <q-btn
          flat
          dense
          round
          icon="close"
          :aria-label="t('common.close')"
          :disable="loading"
          @click="emit('update:modelValue', false)"
        />
      </q-card-section>

      <q-separator />

      <q-card-section class="filter-dialog__body">
        <div v-if="error" role="alert" class="preference-error q-pa-sm">
          {{ t('ui.loadError') }}
        </div>
        <slot />
      </q-card-section>

      <q-separator />

      <q-card-actions align="right" class="filter-dialog__actions">
        <q-btn
          flat
          :label="t('dashboard.filters.reset')"
          :disable="loading"
          @click="emit('reset')"
        />
        <q-btn
          flat
          :label="t('common.cancel')"
          :disable="loading"
          @click="emit('update:modelValue', false)"
        />
        <q-btn
          unelevated
          color="primary"
          :label="t('dashboard.filters.apply')"
          :loading="loading"
          @click="emit('apply')"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

interface Props {
  modelValue: boolean;
  title: string;
  loading?: boolean;
  error?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'apply'): void;
  (e: 'reset'): void;
}>();

const { t } = useI18n();
</script>

<style scoped lang="scss">
.filter-dialog {
  width: 480px;
  max-width: 95vw;
  background: var(--card-bg);
  color: var(--text-color);
}

.filter-dialog__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.filter-dialog__title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-color);
}

.filter-dialog__body {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-height: 70vh;
  overflow-y: auto;
}

.filter-dialog__actions {
  padding: 8px 16px;
}
</style>
