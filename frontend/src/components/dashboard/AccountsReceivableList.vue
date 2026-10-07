<template>
  <div v-if="rows.length === 0" class="chart-empty">{{ emptyLabel }}</div>
  <q-list v-else separator>
    <q-item v-for="(row, idx) in rows" :key="`ar-row-${idx}`">
      <q-item-section>
        <q-item-label>{{ row.client_name }}</q-item-label>
        <q-item-label caption class="text-muted">
          {{ oldestDueDateLabel }}: {{ formatDueDate(row.oldest_due_date) }}
        </q-item-label>
      </q-item-section>
      <q-item-section side class="text-right">
        <CurrencyAmountValue
          :amount="row.outstanding_balance"
          :currency-type="row.currency_type"
          :show-currency="false"
        />
        <div class="text-caption text-muted">{{ row.currency_type }}</div>
      </q-item-section>
    </q-item>
  </q-list>
</template>

<script setup lang="ts">
import type { AccountsReceivableRow } from 'src/types/dashboard';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import CurrencyAmountValue from './CurrencyAmountValue.vue';

interface Props {
  rows: AccountsReceivableRow[];
  emptyLabel: string;
  oldestDueDateLabel: string;
}

defineProps<Props>();

const preferencesStore = useDisplayPreferences();

const formatDueDate = (value: string | null): string => {
  if (value === null) {
    return '—';
  }
  return formatDateByPreferences(value, {
    timezone: preferencesStore.timezone,
    dateFormat: preferencesStore.dateFormat,
    fallback: '—',
  });
};
</script>

<style scoped lang="scss">
.chart-empty {
  min-height: 160px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}

.text-muted {
  color: var(--text-muted);
}
</style>
