<template>
  <q-card class="kpi-card">
    <q-card-section>
      <div class="kpi-head">
        <q-icon :name="icon" size="22px" :class="['kpi-icon', iconClass]" />
        <span class="kpi-title">{{ title }}</span>
      </div>
      <div class="kpi-values">
        <slot name="values">
          <div class="kpi-empty">—</div>
        </slot>
      </div>
      <div class="kpi-caption">{{ caption }}</div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
export type KpiIconVariant = 'default' | 'positive' | 'warning' | 'negative' | 'muted';

interface Props {
  title: string;
  caption: string;
  icon: string;
  variant?: KpiIconVariant;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
});

const iconClass = `kpi-icon--${props.variant}`;
</script>

<style scoped lang="scss">
.kpi-card {
  height: 100%;
  border-radius: 12px;
  background: var(--card-bg);
  transition: transform 0.15s ease;
}

.kpi-card:hover {
  transform: translateY(-2px);
}

.kpi-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.kpi-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}

.kpi-icon {
  color: var(--chart-color-1);
}

.kpi-icon--positive {
  color: var(--chart-color-3);
}

.kpi-icon--warning {
  color: var(--chart-color-4);
}

.kpi-icon--negative {
  color: var(--chart-color-5);
}

.kpi-icon--muted {
  color: var(--text-muted);
}

.kpi-values {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 6px;
  min-height: 44px;
}

.kpi-empty {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-muted);
}

.kpi-caption {
  font-size: 12px;
  color: var(--text-muted);
}
</style>
