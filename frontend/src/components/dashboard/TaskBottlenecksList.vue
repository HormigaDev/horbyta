<template>
  <div v-if="rows.length === 0" class="chart-empty">{{ emptyLabel }}</div>
  <div v-else class="bottlenecks-list">
    <div v-for="row in rows" :key="row.project_id" class="bottleneck-row">
      <router-link class="bottleneck-name text-primary" :to="`/projects/${row.project_id}`">{{
        row.project_name
      }}</router-link>
      <div class="bottleneck-badges">
        <q-chip dense outline color="primary" text-color="primary">
          {{ row.total_tasks }} — {{ labels.total }}
        </q-chip>
        <q-chip dense color="grey-7" text-color="white">
          {{ row.tasks_todo }} — {{ labels.todo }}
        </q-chip>
        <q-chip dense color="primary" text-color="white">
          {{ row.tasks_in_progress }} — {{ labels.inProgress }}
        </q-chip>
        <q-chip dense color="warning" text-color="white">
          {{ row.tasks_in_review }} — {{ labels.review }}
        </q-chip>
        <q-chip dense :color="row.overdue_tasks > 0 ? 'negative' : 'grey-6'" text-color="white">
          {{ row.overdue_tasks }} — {{ labels.overdue }}
        </q-chip>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { TaskBottleneckRow } from 'src/types/dashboard';

export interface TaskBottlenecksLabels {
  total: string;
  todo: string;
  inProgress: string;
  review: string;
  overdue: string;
}

interface Props {
  rows: TaskBottleneckRow[];
  labels: TaskBottlenecksLabels;
  emptyLabel: string;
}

defineProps<Props>();
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

.bottlenecks-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.bottleneck-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--bg-hover);
}

.bottleneck-name {
  font-weight: 600;
  color: var(--text-color);
}

.bottleneck-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
</style>
