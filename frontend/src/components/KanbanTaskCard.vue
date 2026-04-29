<template>
  <q-card
    class="kanban-task-card"
    flat
    bordered
    draggable="true"
    @dragstart="onDragStart"
    @dragend="$emit('drag-end')"
  >
    <q-card-section class="q-pa-sm">
      <div class="row items-start no-wrap q-gutter-sm task-header">
        <div class="col">
          <div class="text-subtitle2 task-title">{{ task.title }}</div>
        </div>
        <div class="row items-center no-wrap q-gutter-xs">
          <div class="text-caption task-muted task-number">
            {{ normalizeId(task.record_number) }}
          </div>
          <q-btn
            flat
            round
            dense
            size="sm"
            icon="open_in_new"
            :to="`/tasks/${task.id}`"
            @dragstart.stop
            @click.stop
          >
            <q-tooltip>{{ $t('common.details') }}</q-tooltip>
          </q-btn>
        </div>
      </div>

      <div v-if="task.project_name || task.milestone_name" class="text-caption task-muted q-mt-sm">
        <div v-if="task.project_name">{{ task.project_name }}</div>
        <div v-if="task.milestone_name">{{ task.milestone_name }}</div>
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import type { KanbanTask } from 'src/types/kanban';
import { normalizeId } from 'src/utils/normalizeId';

const props = defineProps<{
  task: KanbanTask;
}>();

const emit = defineEmits<{
  'drag-start': [task: KanbanTask];
  'drag-end': [];
}>();

const onDragStart = (event: DragEvent) => {
  event.dataTransfer?.setData('text/plain', props.task.id);
  event.dataTransfer?.setData('application/x-task-id', props.task.id);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
  emit('drag-start', props.task);
};
</script>

<style scoped>
.kanban-task-card {
  background: var(--bg-hover) !important;
  cursor: grab;
  border-radius: 8px;
}

.kanban-task-card:active {
  cursor: grabbing;
}

.task-title {
  line-height: 1.25;
  overflow-wrap: anywhere;
}

.task-muted {
  color: var(--text-muted);
}

.task-header {
  min-height: 20px;
}

.task-number {
  flex: 0 0 auto;
  line-height: 1.25;
}
</style>
