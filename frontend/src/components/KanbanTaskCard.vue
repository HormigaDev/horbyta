<template>
  <q-card
    class="kanban-task-card"
    flat
    bordered
    :draggable="!busy"
    :aria-busy="busy"
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
            :aria-label="$t('common.details')"
            @dragstart.stop
            @click.stop
          >
            <q-tooltip>{{ $t('common.details') }}</q-tooltip>
          </q-btn>
          <q-btn
            flat
            round
            dense
            icon="more_horiz"
            :aria-label="$t('ui.move')"
            :disable="busy"
            @dragstart.stop
          >
            <q-menu
              ><q-list style="min-width: 180px"
                ><q-item-label header>{{ $t('ui.move') }}</q-item-label
                ><q-item
                  v-for="column in columns"
                  :key="column.id"
                  clickable
                  v-close-popup
                  :disable="column.id === task.status"
                  @click="$emit('move', task.id, column.id)"
                  ><q-item-section>{{ column.title }}</q-item-section
                  ><q-item-section side v-if="column.id === task.status"
                    ><q-icon name="check" /></q-item-section></q-item></q-list
            ></q-menu>
          </q-btn>
        </div>
      </div>
      <div class="row items-center q-gutter-xs q-mt-sm">
        <span class="task-priority" :class="{ 'task-priority--high': task.priority >= 3 }"
          ><q-icon name="flag" size="14px" /> {{ priorityLabel }}</span
        >
        <span v-if="task.due_date" class="task-due" :class="{ 'task-due--overdue': overdue }"
          ><q-icon name="event" size="14px" /> {{ dueLabel }}
          <span v-if="overdue">— {{ $t('ui.overdue') }}</span></span
        >
      </div>
      <div v-if="task.project_name || task.milestone_name" class="text-caption task-muted q-mt-sm">
        <div v-if="task.project_name">{{ task.project_name }}</div>
        <div v-if="task.milestone_name">{{ task.milestone_name }}</div>
      </div>
    </q-card-section>
  </q-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { TaskStatus } from 'src/types/task';
import type { KanbanColumnDefinition } from 'src/types/kanban';
import type { KanbanTask } from 'src/types/kanban';
import { normalizeId } from 'src/utils/normalizeId';

const props = defineProps<{
  task: KanbanTask;
  columns: KanbanColumnDefinition[];
  busy?: boolean;
}>();

const emit = defineEmits<{
  move: [taskId: string, status: TaskStatus];
  'drag-start': [task: KanbanTask];
  'drag-end': [];
}>();

const { t } = useI18n();
const preferences = useUserPreferencesStore();
const priorityLabel = computed(() => t('tasks.form.priority.' + props.task.priority));
const dueLabel = computed(() =>
  props.task.due_date
    ? formatDateByPreferences(props.task.due_date, {
        timezone: preferences.timezone,
        dateFormat: preferences.dateFormat,
      })
    : '',
);
const overdue = computed(
  () =>
    !!props.task.due_date &&
    props.task.status !== 'done' &&
    new Date(props.task.due_date).getTime() < Date.now(),
);
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
  background: var(--surface-panel);
  box-shadow: var(--shadow-panel);
  border-color: var(--border-subtle);
  cursor: grab;
  border-radius: 12px;
}

.kanban-task-card:active {
  cursor: grabbing;
}

.task-title {
  line-height: 1.5;
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

<style scoped>
.task-priority,
.task-due {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}
.task-priority {
  background: var(--surface-hover);
  padding: 3px 7px;
  border-radius: 5px;
}
.task-priority--high,
.task-due--overdue {
  color: var(--error-text);
}
.task-priority--high {
  background: var(--error-bg);
}
.kanban-task-card .q-card__section {
  padding: 14px;
}
</style>

<style scoped>
.task-header {
  flex-wrap: wrap !important;
}
.task-header > .col {
  flex-basis: 100%;
}
.task-header > .row {
  order: -1;
  width: 100%;
  justify-content: flex-end;
}
.task-header .task-number {
  margin-right: auto;
}
</style>
