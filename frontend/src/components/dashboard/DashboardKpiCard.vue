<template>
  <q-card class="kpi-card" flat bordered>
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
      <q-btn
        v-if="actionTo && actionLabel"
        flat
        dense
        color="primary"
        class="q-mt-sm"
        :to="actionTo"
        :label="actionLabel"
        icon-right="arrow_forward"
      />
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';

export type KpiIconVariant = 'default' | 'positive' | 'warning' | 'negative' | 'muted';

interface Props {
  title: string;
  caption: string;
  icon: string;
  variant?: KpiIconVariant;
  actionTo?: string;
  actionLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
});

const iconClass = computed(() => `kpi-icon--${props.variant}`);
</script>

<style scoped lang="scss">
.kpi-card {
  height: 100%;
  border-radius: 12px;
  background: var(--card-bg);
  border-color: var(--border-subtle);
  box-shadow: var(--shadow-panel);
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
  line-height: 1.5;
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
  font-size: 13px;
  color: var(--text-muted);
}
</style>
