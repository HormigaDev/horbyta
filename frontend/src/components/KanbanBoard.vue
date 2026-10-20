<template>
  <div>
    <q-select
      v-if="$q.screen.lt.sm"
      :model-value="displayedColumns[0]?.id ?? null"
      @update:model-value="(id: string) => (mobileStatus = id)"
      :options="
        columns.map((c) => ({
          label: `${c.title} (${tasksByStatus[c.id]?.length ?? 0})`,
          value: c.id,
        }))
      "
      emit-value
      map-options
      outlined
      :label="$t('tasks.form.statusLabel')"
      class="q-mb-md"
    />
    <div class="kanban-board" :style="{ '--kanban-columns': displayedColumns.length }">
      <KanbanColumn
        v-for="column in displayedColumns"
        :key="column.id"
        :column="column"
        :columns="columns"
        :busy="busy"
        @move="(id, status) => handleDrop(status, id)"
        :tasks="tasksByStatus[column.id] ?? []"
        @drop="handleDrop"
        @task-drag-start="onTaskDragStart"
        @task-drag-end="clearDragState"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useQuasar } from 'quasar';
import KanbanColumn from './KanbanColumn.vue';
import type { KanbanColumnDefinition, KanbanTask } from 'src/types/kanban';

const props = defineProps<{
  columns: KanbanColumnDefinition[];
  tasks: KanbanTask[];
  busy?: boolean;
}>();

const emit = defineEmits<{
  'task-drop': [taskId: string, newColumnId: string];
}>();

const $q = useQuasar();
const mobileStatus = ref<string | null>(null);
const displayedColumns = computed(() => {
  if (!$q.screen.lt.sm) return props.columns;
  const selected = props.columns.find((c) => c.id === mobileStatus.value) ?? props.columns[0];
  return selected ? [selected] : [];
});
const draggedTask = ref<KanbanTask | null>(null);

const tasksByStatus = computed(() => {
  return props.tasks.reduce(
    (result, task) => {
      result[task.status_id] = [...(result[task.status_id] ?? []), task];
      return result;
    },
    {} as Record<string, KanbanTask[]>,
  );
});

const onTaskDragStart = (task: KanbanTask) => {
  draggedTask.value = task;
};

const clearDragState = () => {
  draggedTask.value = null;
};

const findTask = (taskId: string | null) => {
  if (!taskId) return draggedTask.value;
  return props.tasks.find((task) => task.id === taskId) ?? draggedTask.value;
};

const handleDrop = (newColumnId: string, taskId: string | null) => {
  const task = findTask(taskId);
  if (!task || task.status_id === newColumnId) {
    clearDragState();
    return;
  }

  emit('task-drop', task.id, newColumnId);
  clearDragState();
};
</script>

<style scoped>
.kanban-board {
  display: grid;
  gap: 16px;
  /* Tantas columnas como estados; si no caben, el tablero se desplaza en horizontal. */
  grid-template-columns: repeat(var(--kanban-columns, 4), minmax(220px, 1fr));
  min-width: 100%;
  overflow-x: auto;
  padding-bottom: 4px;
}

@media (max-width: 599px) {
  .kanban-board {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
