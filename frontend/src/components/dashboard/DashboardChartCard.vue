<template>
  <q-card class="chart-card">
    <q-card-section>
      <div class="chart-head">
        <div>
          <div class="chart-title">{{ title }}</div>
          <div v-if="caption" class="chart-caption">{{ caption }}</div>
        </div>
        <slot name="actions" />
      </div>
      <div
        :class="[
          'chart-container',
          short ? 'chart-container--short' : '',
          autoHeight ? 'chart-container--auto' : '',
        ]"
      >
        <slot />
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
interface Props {
  title: string;
  caption?: string;
  short?: boolean;
  autoHeight?: boolean;
}

withDefaults(defineProps<Props>(), {
  caption: '',
  short: false,
  autoHeight: false,
});
</script>

<style scoped lang="scss">
.chart-card {
  height: 100%;
  border-radius: 12px;
  background: var(--card-bg);
}

.chart-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
  gap: 8px;
}

.chart-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-color);
}

.chart-caption {
  font-size: 12px;
  color: var(--text-muted);
}

.chart-container {
  height: 320px;
  min-height: 320px;
  width: 100%;
}

.chart-container--short {
  height: 260px;
  min-height: 260px;
}

.chart-container--auto {
  height: auto;
}

@media (max-width: 600px) {
  .chart-container {
    height: 280px;
    min-height: 280px;
  }
  .chart-container--short {
    height: 220px;
    min-height: 220px;
  }
  .chart-container--auto {
    height: auto;
  }
}
</style>
