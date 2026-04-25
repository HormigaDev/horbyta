<template>
  <div v-if="rows.length === 0" class="chart-empty">{{ emptyLabel }}</div>
  <div v-else class="status-list">
    <div v-for="row in rows" :key="row.status" class="status-row">
      <div class="status-row__left">
        <div class="status-row__title">{{ labelForStatus(row.status) }}</div>
        <div class="status-row__caption">{{ helperForStatus(row.status) }}</div>
      </div>
      <q-badge rounded :color="badgeColor(row.status)" class="status-row__badge">
        {{ row.total_requests }}
      </q-badge>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ClientChangeRequestStatusRow } from 'src/types/dashboard';
import type { ChangeRequestStatus } from 'src/types/changeRequest';

interface Props {
  rows: ClientChangeRequestStatusRow[];
  emptyLabel: string;
  labelForStatus: (status: ChangeRequestStatus) => string;
  helperForStatus: (status: ChangeRequestStatus) => string;
}

defineProps<Props>();

const badgeColor = (status: ChangeRequestStatus) => {
  switch (status) {
    case 'approved':
    case 'implemented':
      return 'positive';
    case 'declined':
    case 'cancelled':
      return 'negative';
    case 'assessed':
      return 'warning';
    default:
      return 'primary';
  }
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

.status-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.status-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--bg-hover);
}

.status-row__left {
  min-width: 0;
}

.status-row__title {
  font-weight: 600;
  color: var(--text-color);
}

.status-row__caption {
  font-size: 12px;
  color: var(--text-muted);
}

.status-row__badge {
  min-width: 28px;
  justify-content: center;
}
</style>
