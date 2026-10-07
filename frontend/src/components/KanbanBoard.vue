<template>
  <div>
    <q-select
      v-if="$q.screen.lt.sm"
      v-model="mobileStatus"
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
    <div class="kanban-board">
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
import type { TaskStatus } from 'src/types/task';

const props = defineProps<{
  columns: KanbanColumnDefinition[];
  tasks: KanbanTask[];
  busy?: boolean;
}>();

const emit = defineEmits<{
  'task-drop': [taskId: string, newColumnId: TaskStatus];
}>();

const $q = useQuasar();
const mobileStatus = ref<TaskStatus>('todo');
const displayedColumns = computed(() =>
  $q.screen.lt.sm ? props.columns.filter((c) => c.id === mobileStatus.value) : props.columns,
);
const draggedTask = ref<KanbanTask | null>(null);

const tasksByStatus = computed(() => {
  return props.tasks.reduce(
    (result, task) => {
      result[task.status] = [...(result[task.status] ?? []), task];
      return result;
    },
    {} as Record<TaskStatus, KanbanTask[]>,
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

const handleDrop = (newColumnId: TaskStatus, taskId: string | null) => {
  const task = findTask(taskId);
  if (!task || task.status === newColumnId) {
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
  grid-template-columns: repeat(4, minmax(0, 1fr));
  min-width: 100%;
}

@media (max-width: 1023px) {
  .kanban-board {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 599px) {
  .kanban-board {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
