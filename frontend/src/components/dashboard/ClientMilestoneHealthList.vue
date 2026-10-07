<template>
  <div v-if="rows.length === 0" class="chart-empty">{{ emptyLabel }}</div>
  <div v-else class="health-list">
    <div v-for="row in rows" :key="row.project_id" class="health-row">
      <div class="health-row__head">
        <router-link
          class="health-row__name text-primary"
          :to="{ path: '/client/milestones', query: { project_id: row.project_id } }"
          >{{ row.project_name }}</router-link
        >
        <div class="health-row__date">
          <span class="text-caption">{{ nextDueDateLabel }}</span>
          <span>{{ row.next_due_date ? formatDate(row.next_due_date) : emptyValue }}</span>
        </div>
      </div>

      <div class="health-row__chips">
        <q-chip dense outline color="primary" text-color="primary">
          {{ row.total_milestones }} · {{ totalLabel }}
        </q-chip>
        <q-chip dense color="positive" text-color="white">
          {{ row.completed_milestones }} · {{ completedLabel }}
        </q-chip>
        <q-chip dense color="warning" text-color="white">
          {{ row.upcoming_milestones }} · {{ upcomingLabel }}
        </q-chip>
        <q-chip
          dense
          :color="row.overdue_milestones > 0 ? 'negative' : 'grey-6'"
          text-color="white"
        >
          {{ row.overdue_milestones }} · {{ overdueLabel }}
        </q-chip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ClientMilestoneHealthRow } from 'src/types/dashboard';

interface Props {
  rows: ClientMilestoneHealthRow[];
  emptyLabel: string;
  totalLabel: string;
  completedLabel: string;
  upcomingLabel: string;
  overdueLabel: string;
  nextDueDateLabel: string;
  emptyValue: string;
}

defineProps<Props>();
const preferencesStore = useDisplayPreferences();

const formatDate = (value: string) =>
  formatDateByPreferences(value, {
    timezone: preferencesStore.timezone,
    dateFormat: preferencesStore.dateFormat,
    fallback: value,
  });
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

.health-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.health-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--bg-hover);
}

.health-row__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.health-row__name {
  font-weight: 600;
  color: var(--text-color);
}

.health-row__date {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  color: var(--text-color);
}

.health-row__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

@media (max-width: 600px) {
  .health-row__head {
    flex-direction: column;
  }

  .health-row__date {
    align-items: flex-start;
  }
}
</style>
