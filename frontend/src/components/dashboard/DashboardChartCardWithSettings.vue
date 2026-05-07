<template>
  <DashboardChartCard :title="title" :caption="caption" :short="short" :auto-height="autoHeight">
    <template #actions>
      <q-btn
        flat
        dense
        round
        icon="tune"
        size="sm"
        :aria-label="t('dashboard.filters.openSettings')"
        @click="emit('open')"
      >
        <q-tooltip>{{ t('dashboard.filters.openSettings') }}</q-tooltip>
      </q-btn>
    </template>

    <slot />

    <ChartFilterDialog
      :model-value="open"
      :title="filtersTitle"
      @update:model-value="(value) => (value ? emit('open') : emit('close'))"
      @apply="emit('apply')"
      @reset="emit('reset')"
    >
      <slot name="filters" />
    </ChartFilterDialog>
  </DashboardChartCard>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import DashboardChartCard from './DashboardChartCard.vue';
import ChartFilterDialog from './ChartFilterDialog.vue';

interface Props {
  title: string;
  caption?: string;
  short?: boolean;
  autoHeight?: boolean;
  filtersTitle: string;
  open: boolean;
}

withDefaults(defineProps<Props>(), {
  caption: '',
  short: false,
  autoHeight: false,
});

const emit = defineEmits<{
  (e: 'open'): void;
  (e: 'close'): void;
  (e: 'apply'): void;
  (e: 'reset'): void;
}>();

const { t } = useI18n();
</script>
