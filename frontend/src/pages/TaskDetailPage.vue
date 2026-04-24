<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>
    <div class="col-12 col-md-8">
      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="flex justify-between">
            <div class="text-h5 text-weight-bold q-mb-xs" style="color: var(--text-color)">
              <q-icon name="checklist" size="lg" />
              {{ task?.title ? `${task.title}` : t('common.loading') }}
            </div>
            <div class="text-h6 flex items-center" style="color: var(--text-muted)">
              <q-icon
                :name="idCopied ? 'check' : 'content_copy'"
                size="xs"
                class="q-mr-sm cursor-pointer"
                style="color: var(--text-color)"
                @click="copyToClipboard(task?.record_number.toString() || '')"
              >
                <q-tooltip>{{
                  idCopied ? $t('common.copied') : $t('common.copyToClipboard')
                }}</q-tooltip>
              </q-icon>
              {{ task ? normalizeId(task.record_number) : '' }}
            </div>
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="task">
            {{
              t('tasks.detail.meta', {
                id: task.id,
                created: formatDate(task.created_at),
                updated: formatDate(task.updated_at),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('tasks.detail.specsTitle') }}
          </div>
          <div class="row q-col-gutter-md q-mb-md" v-if="task">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.status') }}
              </div>
              <div style="color: var(--text-color)">{{ statusLabel(task.status) }}</div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.priority') }}
              </div>
              <div style="color: var(--text-color)">
                {{ $t(`tasks.form.priority.${task.priority}`) }}
              </div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.complexity') }}
              </div>
              <div style="color: var(--text-color)">
                {{ $t(`tasks.form.complexity.${task.complexity}`) }}
              </div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.project') }}
              </div>
              <div style="color: var(--text-color)">
                {{ task.project_name || t('tasks.detail.noProject') }}
              </div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('common.milestone') }}
              </div>
              <div style="color: var(--text-color)">
                {{ task.milestone_name || t('tasks.detail.noMilestone') }}
              </div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('tasks.list.filter.dueDate') }}
              </div>
              <div style="color: var(--text-color)">
                {{ task.due_date ? formatDate(task.due_date) : t('tasks.detail.noDate') }}
              </div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-tabs
          v-model="detailsTab"
          dense
          align="justify"
          indicator-color="primary"
          active-color="primary"
        >
          <q-tab name="description" :label="t('tasks.detail.tabs.description')" />
          <q-tab name="attachments" :label="t('tasks.detail.tabs.attachments')" />
          <q-tab name="time_entries" :label="t('tasks.detail.tabs.timeEntries')" />
        </q-tabs>

        <q-separator />

        <q-tab-panels v-model="detailsTab" animated style="background-color: var(--card-bg)">
          <q-tab-panel name="description">
            <MarkdownRenderer
              :model-value="task?.description || t('tasks.detail.emptyDescription')"
            />
          </q-tab-panel>

          <q-tab-panel name="attachments">
            <TaskAttachmentsTab v-if="task" :task-id="task.id" />
          </q-tab-panel>

          <q-tab-panel name="time_entries">
            <TaskTimeEntriesTab v-if="task" :task="task" ref="timeEntriesTabRef" />
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>

    <div class="col-12 col-md-2" style="position: sticky; top: 80px">
      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list separator>
          <q-item clickable v-ripple @click="goBack" class="q-pa-md">
            <q-item-section avatar><q-icon name="arrow_back" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.back') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.status === 'todo'"
            @click="updateTaskStatus('in_progress')"
          >
            <q-item-section avatar><q-icon name="start" /></q-item-section>
            <q-item-section>{{ $t('tasks.detail.action.startTask') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task"
            @click="timeEntriesTabRef?.openManualTimeEntryDialog()"
          >
            <q-item-section avatar><q-icon name="edit_note" /></q-item-section>
            <q-item-section>{{ $t('tasks.detail.action.logManualTime') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && !timeEntriesTabRef?.activeTimerEntry"
            @click="timeEntriesTabRef?.openStartTimerDialog()"
          >
            <q-item-section avatar><q-icon name="timer" /></q-item-section>
            <q-item-section>{{ $t('tasks.detail.action.startTimer') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="timeEntriesTabRef?.activeTimerBelongsToCurrentTask"
            @click="timeEntriesTabRef?.stopActiveTimerForCurrentTask()"
          >
            <q-item-section avatar><q-icon name="stop_circle" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.stopTimer') }}</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="task">
            <q-item-section avatar><q-icon name="edit" color="primary" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.edit') }}</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-negative q-pa-md" @click="removeTask" v-if="task">
            <q-item-section avatar><q-icon name="delete" color="negative" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import TaskAttachmentsTab from 'src/components/TaskAttachmentsTab.vue';
import TaskTimeEntriesTab from 'src/components/TaskTimeEntriesTab.vue';
import { useUI } from 'src/composables/useUI';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import { TasksService } from 'src/services/tasks';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { normalizeId } from 'src/utils/normalizeId';
import type { TaskDetailDto, TaskStatus } from 'src/types/task';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();

const task = ref<TaskDetailDto | null>(null);
const taskId = String(route.params.id || '');
const detailsTab = ref<'description' | 'attachments' | 'time_entries'>('description');
const timeEntriesTabRef = ref<InstanceType<typeof TaskTimeEntriesTab> | null>(null);

const formatDate = (val: string) =>
  formatDateByPreferences(val, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
  });

const statusLabel = (status: TaskStatus) => {
  const map: Record<string, string> = {
    todo: 'TODO',
    in_progress: 'In Progress',
    review: 'Review',
    done: 'Done',
  };
  return map[status] || status;
};

const loadTask = async () => {
  showLoading();
  try {
    task.value = await TasksService.getById(taskId);
  } catch (err) {
    notifyError(t('tasks.detail.loadError'), err);
    router.push('/tasks');
  } finally {
    hideLoading();
  }
};

const updateTaskStatus = async (status: TaskStatus) => {
  if (!task.value) return;
  showLoading();
  try {
    await TasksService.changeStatus(task.value.id, status);
    task.value.status = status;
    notify(t('common.success'), 'positive');
  } catch (err) {
    notifyError(t('common.error'), err);
  } finally {
    hideLoading();
  }
};

const removeTask = async () => {
  if (!(await confirmDelete(t('tasks.detail.deleteTarget')))) return;
  showLoading();
  try {
    await TasksService.delete(taskId);
    router.push('/tasks');
  } catch (err) {
    notifyError(t('common.error'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => router.push('/tasks');
const goToEdit = () => router.push(`/tasks/${taskId}/edit`);

onMounted(() => loadTask());
</script>
