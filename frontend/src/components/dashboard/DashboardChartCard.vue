<template>
  <q-card class="chart-card" flat bordered>
    <q-card-section>
      <div class="chart-head">
        <div>
          <div class="chart-title" role="heading" aria-level="2">{{ title }}</div>
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
  background: var(--surface-panel);
  box-shadow: var(--shadow-panel);
  border-color: var(--border-subtle);
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
  width: 100%;
  min-width: 0;
  --chart-height: 320px;
}
.chart-container--short {
  --chart-height: 260px;
}
@media (max-width: 600px) {
  .chart-head {
    flex-wrap: wrap;
  }
  .chart-container {
    --chart-height: 260px;
  }
  .chart-container--short {
    --chart-height: 240px;
  }
}
</style>
