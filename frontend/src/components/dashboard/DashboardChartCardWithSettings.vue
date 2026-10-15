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
        :disable="loading"
        @click="emit('open')"
      >
        <q-tooltip>{{ t('dashboard.filters.openSettings') }}</q-tooltip>
        <q-badge v-if="filterCount" floating color="primary">{{ filterCount }}</q-badge>
      </q-btn>
    </template>

    <div v-if="error" class="preference-error q-pa-sm q-mb-sm" role="alert">
      {{ t('ui.loadError') }}
      <q-btn flat dense :label="t('ui.retry')" :loading="loading" @click="emit('retry')" />
    </div>
    <div :aria-busy="loading">
      <ContentSkeleton v-if="loading" variant="chart" /><slot v-else />
    </div>

    <ChartFilterDialog
      :model-value="open"
      :title="filtersTitle"
      :loading="loading"
      :error="error"
      @update:model-value="(value) => (value ? emit('open') : emit('close'))"
      @apply="emit('apply')"
      @reset="emit('reset')"
    >
      <slot name="filters" />
    </ChartFilterDialog>
  </DashboardChartCard>
</template>

<script setup lang="ts">
import ContentSkeleton from 'src/components/ContentSkeleton.vue';
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
  loading?: boolean;
  error?: boolean;
  filterCount?: number;
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
  (e: 'retry'): void;
}>();

const { t } = useI18n();
</script>
