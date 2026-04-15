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
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{
              task?.title ? `${task.title} - ${normalizeId(task.task_number)}` : t('common.loading')
            }}
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
                {{
                  task.due_date
                    ? formatDate(task.due_date, t('tasks.detail.noDate'))
                    : t('tasks.detail.noDate')
                }}
              </div>
            </div>
            <div class="col-12 col-md-6" v-if="task.parent_id">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('tasks.detail.blockParent') }}
              </div>
              <div style="color: var(--text-color)">
                {{ task.block_parent ? t('common.yes') : t('common.no') }}
              </div>
            </div>
          </div>

          <div v-if="task">
            <div class="text-caption q-mb-xs" style="color: var(--text-muted)">
              {{ t('tasks.detail.tagsTitle') }}
            </div>
            <div class="row q-gutter-xs">
              <q-chip
                v-for="tag in task.tags"
                :key="tag.id"
                dense
                :style="`background-color: ${tag.color};
                  border: none;
                  color: ${getTagTextContrastColor(tag.color)};`"
              >
                {{ tag.name }}
              </q-chip>
              <span v-if="task.tags.length === 0" style="color: var(--text-muted)">{{
                t('tasks.detail.noTags')
              }}</span>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ t('tasks.detail.descriptionTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer
              :model-value="task?.description || t('tasks.detail.emptyDescription')"
            />
          </div>
        </q-card-section>
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
            <q-item-section avatar>
              <q-icon name="arrow_back" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">{{
              t('tasks.detail.action.back')
            }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.status === 'todo'"
            @click="() => updateTaskStatus()"
          >
            <q-item-section avatar>
              <q-icon name="start" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ $t('tasks.detail.action.startTask') }}
            </q-item-section>
          </q-item>
          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.status !== 'todo'"
            @click="openChangeStatusDialog"
          >
            <q-item-section avatar>
              <q-icon name="sync" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">
              {{ $t('tasks.detail.action.changeStatus') }}
            </q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="task">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ t('tasks.detail.action.edit') }}</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-negative q-pa-md" @click="removeTask" v-if="task">
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>{{ t('tasks.detail.action.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useUI } from 'src/composables/useUI';
import { TasksService } from 'src/services/tasks';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { useQuasar } from 'quasar';
import TaskChangeStatusDialog from 'src/components/TaskChangeStatusDialog.vue';
import type { TaskDetailDto, TaskStatus } from 'src/types/task';
import { getTagTextContrastColor } from 'src/utils/tagColorContrast';
import { normalizeId } from 'src/utils/normalizeId';

const $q = useQuasar();
const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const task = ref<TaskDetailDto | null>(null);
const taskId = String(route.params.id || '');

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const statusLabel = (status: TaskStatus) => {
  switch (status) {
    case 'todo':
      return t('tasks.detail.status.todo');
    case 'in_progress':
      return t('tasks.detail.status.in_progress');
    case 'review':
      return t('tasks.detail.status.review');
    case 'done':
      return t('tasks.detail.status.done');
    default:
      return status;
  }
};

const updateTaskStatus = async (taskStatus: TaskStatus = 'in_progress') => {
  if (!task.value) return;
  showLoading(t('tasks.detail.updatingTask'));

  try {
    await TasksService.changeStatus(task.value.id, taskStatus);
    task.value.status = taskStatus;
    notify(t('tasks.detail.taskStartedSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('tasks.detail.changeStatusError'), err);
  } finally {
    hideLoading();
  }
};

const openChangeStatusDialog = () => {
  $q.dialog({
    component: TaskChangeStatusDialog,
  }).onOk((selectedValue: TaskStatus) => {
    if (!task.value) return;
    updateTaskStatus(selectedValue);
  });
};

const loadTask = async () => {
  showLoading(t('tasks.detail.loadTask'));
  try {
    task.value = await TasksService.getById(taskId);
  } catch (err: unknown) {
    notifyError(t('tasks.detail.loadError'), err);
    void router.push('/tasks');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/tasks');
};

const goToEdit = () => {
  if (!task.value) return;
  void router.push(`/tasks/${task.value.id}/edit`);
};

const removeTask = async () => {
  if (!task.value) return;

  const confirmed = await confirmDelete(t('tasks.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('tasks.detail.deleting'));
  try {
    await TasksService.delete(task.value.id);
    notify(t('tasks.detail.deleteSuccess'), 'positive');
    void router.push('/tasks');
  } catch (err: unknown) {
    notifyError(t('tasks.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

onMounted(() => {
  void loadTask();
});
</script>
