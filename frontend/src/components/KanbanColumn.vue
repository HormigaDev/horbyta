<template>
  <section class="kanban-column" @dragover.prevent @drop="onDrop">
    <div class="row items-center justify-between q-mb-sm">
      <div class="text-subtitle1 text-weight-medium">{{ column.title }}</div>
      <q-badge class="kanban-count" rounded>{{ tasks.length }}</q-badge>
    </div>

    <div class="column q-gutter-sm">
      <KanbanTaskCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        @drag-start="$emit('task-drag-start', task)"
        @drag-end="$emit('task-drag-end')"
      />

      <div v-if="tasks.length === 0" class="empty-column text-caption">
        {{ $t('board.emptyColumn') }}
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import KanbanTaskCard from './KanbanTaskCard.vue';
import type { KanbanColumnDefinition, KanbanTask } from 'src/types/kanban';

const props = defineProps<{
  column: KanbanColumnDefinition;
  tasks: KanbanTask[];
}>();

const emit = defineEmits<{
  drop: [columnId: KanbanColumnDefinition['id'], taskId: string | null];
  'task-drag-start': [task: KanbanTask];
  'task-drag-end': [];
}>();

const onDrop = (event: DragEvent) => {
  const taskId =
    event.dataTransfer?.getData('application/x-task-id') ||
    event.dataTransfer?.getData('text/plain') ||
    null;

  emit('drop', props.column.id, taskId);
};
</script>

<style scoped>
.kanban-column {
  background: var(--card-bg);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  min-height: 320px;
  padding: 12px;
}

.kanban-count {
  background: var(--bg-hover);
  color: var(--text-color);
}

.empty-column {
  border: 1px dashed var(--border-color);
  border-radius: 8px;
  color: var(--text-muted);
  padding: 16px;
  text-align: center;
}
</style>
