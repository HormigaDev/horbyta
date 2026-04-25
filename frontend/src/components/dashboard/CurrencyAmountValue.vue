<template>
  <div class="kpi-value">
    <span class="kpi-amount" :class="{ 'kpi-amount--big': big }">{{ formatted }}</span>
    <span v-if="showCurrency" class="kpi-currency">{{ currencyType }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { CURRENCY_SYMBOLS } from 'src/data/constants';

interface Props {
  amount: string | number;
  currencyType: string;
  showCurrency?: boolean;
  big?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showCurrency: true,
  big: false,
});

const formatted = computed<string>(() => {
  const num = typeof props.amount === 'number' ? props.amount : Number(props.amount);
  if (!Number.isFinite(num)) {
    return '—';
  }
  const symbol = CURRENCY_SYMBOLS[props.currencyType] ?? `${props.currencyType} `;
  return `${symbol} ${num.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
});
</script>

<style scoped lang="scss">
.kpi-value {
  display: flex;
  align-items: baseline;
  gap: 6px;
  color: var(--text-color);
}

.kpi-amount {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-color);
}

.kpi-amount--big {
  font-size: 32px;
}

.kpi-currency {
  font-size: 12px;
  color: var(--text-muted);
}

@media (max-width: 600px) {
  .kpi-amount {
    font-size: 20px;
  }
  .kpi-amount--big {
    font-size: 28px;
  }
}
</style>
