<template>
  <section
    class="kanban-column"
    :data-status="column.category"
    :style="{ borderTopColor: column.color }"
    :class="{ 'kanban-column--target': dragOver }"
    @dragleave="dragOver = false"
    @dragover.prevent="dragOver = true"
    @drop="onDrop"
  >
    <div class="row items-center justify-between q-mb-sm">
      <div class="text-subtitle1 text-weight-medium">
        <q-icon :name="statusIcon" size="18px" class="q-mr-sm" :style="{ color: column.color }" />{{
          column.title
        }}
      </div>
      <q-badge class="kanban-count" rounded>{{ tasks.length }}</q-badge>
    </div>

    <div class="column q-gutter-sm">
      <KanbanTaskCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        :columns="columns"
        :busy="busy"
        @move="(id, status) => $emit('move', id, status)"
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
import { ref, computed } from 'vue';
import KanbanTaskCard from './KanbanTaskCard.vue';
import type { KanbanColumnDefinition, KanbanTask } from 'src/types/kanban';

const props = defineProps<{
  column: KanbanColumnDefinition;
  tasks: KanbanTask[];
  columns: KanbanColumnDefinition[];
  busy?: boolean;
}>();

const emit = defineEmits<{
  move: [taskId: string, statusId: string];
  drop: [columnId: KanbanColumnDefinition['id'], taskId: string | null];
  'task-drag-start': [task: KanbanTask];
  'task-drag-end': [];
}>();

const dragOver = ref(false);
const statusIcon = computed(
  () =>
    ({
      todo: 'radio_button_unchecked',
      in_progress: 'timelapse',
      review: 'rate_review',
      done: 'check_circle_outline',
      cancelled: 'block',
    })[props.column.category],
);
const onDrop = (event: DragEvent) => {
  dragOver.value = false;
  const taskId =
    event.dataTransfer?.getData('application/x-task-id') ||
    event.dataTransfer?.getData('text/plain') ||
    null;

  emit('drop', props.column.id, taskId);
};
</script>

<style scoped>
.kanban-column {
  background: var(--surface-page);
  border: 1px solid var(--border-color);
  border-radius: 14px;
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

<style scoped>
.kanban-column {
  border-top: 3px solid var(--text-secondary);
}
.kanban-column[data-status='in_progress'] {
  border-top-color: var(--info-text);
}
.kanban-column[data-status='review'] {
  border-top-color: var(--warning-text);
}
.kanban-column[data-status='done'] {
  border-top-color: var(--success-text);
}
.kanban-column--target {
  outline: 2px dashed var(--focus-ring);
  background: var(--surface-selected);
}
</style>
