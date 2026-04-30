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
        v-if="task && task.parent_id"
        class="q-mb-md"
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list>
          <q-item clickable v-ripple @click="goToParent" class="q-pa-md">
            <q-item-section avatar>
              <q-icon name="account_tree" color="primary" />
            </q-item-section>
            <q-item-section>
              <q-item-label class="text-caption" style="color: var(--text-muted)">
                {{ t('tasks.detail.parentTitle') }}
              </q-item-label>
              <q-item-label style="color: var(--text-color)">
                {{
                  task.parent_record_number !== null
                    ? `#${normalizeId(task.parent_record_number)} · ${task.parent_title ?? ''}`
                    : (task.parent_title ?? '')
                }}
              </q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-icon name="chevron_right" />
              <q-tooltip>{{ t('tasks.detail.openParent') }}</q-tooltip>
            </q-item-section>
          </q-item>
        </q-list>
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
              <div style="color: var(--text-color)">
                {{ $t(`tasks.form.status.${task.status}`) }}
              </div>
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
          <q-tab
            v-if="hasSubtasks"
            name="subtasks"
            :label="t('tasks.detail.tabs.subtasks')"
          />
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

          <q-tab-panel name="subtasks" v-if="hasSubtasks">
            <q-list separator v-if="subtasks.length > 0">
              <q-item
                v-for="subtask in subtasks"
                :key="subtask.id"
                clickable
                v-ripple
                @click="goToSubtask(subtask.id)"
              >
                <q-item-section avatar>
                  <q-icon name="checklist" />
                </q-item-section>
                <q-item-section>
                  <q-item-label style="color: var(--text-color)">
                    #{{ normalizeId(subtask.record_number) }} · {{ subtask.title }}
                  </q-item-label>
                  <q-item-label caption style="color: var(--text-muted)">
                    {{ $t(`tasks.form.status.${subtask.status}`) }}
                  </q-item-label>
                </q-item-section>
                <q-item-section side>
                  <q-icon name="chevron_right" />
                </q-item-section>
              </q-item>
            </q-list>
            <div v-else class="text-center q-pa-md" style="color: var(--text-muted)">
              {{ t('tasks.detail.subtasksEmpty') }}
            </div>
          </q-tab-panel>
        </q-tab-panels>
      </q-card>
    </div>

    <q-dialog v-model="parentDialogOpen">
      <q-card
        style="
          min-width: 360px;
          background-color: var(--card-bg);
          border: 1px solid var(--border-color);
        "
      >
        <q-card-section>
          <div class="text-h6" style="color: var(--text-color)">
            {{ t('tasks.detail.action.parentDialogTitle') }}
          </div>
        </q-card-section>
        <q-card-section class="q-pt-none">
          <SelectByTerm
            v-model="parentDialogSelection"
            :label="t('tasks.form.parentTaskLabel')"
            :options="parentDialogOptions"
            :loading="isParentDialogLoading"
            @search="onParentDialogSearch"
          />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" v-close-popup color="grey" />
          <q-btn
            unelevated
            color="primary"
            :label="t('tasks.detail.action.parentDialogConfirm')"
            :disable="!parentDialogSelection"
            @click="confirmParentSelection"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

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

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task"
            @click="goToCreateSubtask"
          >
            <q-item-section avatar><q-icon name="subdirectory_arrow_right" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.createSubtask') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.project_id && !task.parent_id"
            @click="openParentDialog"
          >
            <q-item-section avatar><q-icon name="account_tree" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.assignParent') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.project_id && task.parent_id"
            @click="openParentDialog"
          >
            <q-item-section avatar><q-icon name="account_tree" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.changeParent') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && task.parent_id"
            @click="removeParent"
          >
            <q-item-section avatar><q-icon name="link_off" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.removeParent') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && !userPreferencesStore.isPinned(task.id)"
            @click="pinCurrentTask"
          >
            <q-item-section avatar><q-icon name="push_pin" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.pin') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="q-pa-md"
            v-if="task && userPreferencesStore.isPinned(task.id)"
            @click="unpinCurrentTask"
          >
            <q-item-section avatar><q-icon name="push_pin" color="primary" /></q-item-section>
            <q-item-section>{{ t('tasks.detail.action.unpin') }}</q-item-section>
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
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import TaskAttachmentsTab from 'src/components/TaskAttachmentsTab.vue';
import TaskTimeEntriesTab from 'src/components/TaskTimeEntriesTab.vue';
import { useUI } from 'src/composables/useUI';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import { TasksService } from 'src/services/tasks';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import { normalizeId } from 'src/utils/normalizeId';
import type { Filter } from 'src/types';
import type { TaskDetailDto, TaskStatus, TaskSummaryDto } from 'src/types/task';
import type { PinnedTask } from 'src/types/user';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
const { isCopied: idCopied, copyToClipboard } = useCopyClipboard();

const task = ref<TaskDetailDto | null>(null);
const currentTaskId = () => String(route.params.id || '');
const detailsTab = ref<'description' | 'attachments' | 'time_entries' | 'subtasks'>('description');
const timeEntriesTabRef = ref<InstanceType<typeof TaskTimeEntriesTab> | null>(null);

const subtasks = ref<TaskSummaryDto[]>([]);
const hasSubtasks = computed(() => subtasks.value.length > 0);

const parentDialogOpen = ref(false);
const parentDialogSelection = ref<string | null>(null);

const {
  options: parentDialogOptions,
  isLoading: isParentDialogLoading,
  load: loadParentDialogTasks,
  setOptions: setParentDialogOptions,
} = useSelectByTerm<TaskSummaryDto>(TasksService.search, {
  termColumn: 'title',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (item) => ({
    label: `#${normalizeId(item.record_number)} · ${item.title}`,
    value: item.id,
  }),
});

const formatDate = (val: string) =>
  formatDateByPreferences(val, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
  });

const loadSubtasks = async (parentId: string) => {
  try {
    const response = await TasksService.search({
      page: 1,
      limit: 100,
      order_by: 'created_at',
      order_asc: false,
      filters: [{ column: 'parent_id', operator: '=', value: parentId }],
    });
    subtasks.value = response.items;
  } catch (err) {
    subtasks.value = [];
    notifyError(t('tasks.detail.subtasksLoadError'), err);
  }
};

const loadTask = async (id: string) => {
  showLoading();
  try {
    task.value = await TasksService.getById(id);
    await loadSubtasks(id);
    if (detailsTab.value === 'subtasks' && subtasks.value.length === 0) {
      detailsTab.value = 'description';
    }
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
    await TasksService.delete(currentTaskId());
    router.push('/tasks');
  } catch (err) {
    notifyError(t('common.error'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => router.push('/tasks');
const goToEdit = () => router.push(`/tasks/${currentTaskId()}/edit`);
const goToParent = () => {
  if (task.value?.parent_id) {
    void router.push(`/tasks/${task.value.parent_id}`);
  }
};
const goToSubtask = (id: string) => {
  void router.push(`/tasks/${id}`);
};
const goToCreateSubtask = () => {
  if (!task.value) return;
  const query: Record<string, string> = { parentId: task.value.id };
  if (task.value.project_id) {
    query.projectId = task.value.project_id;
  }
  void router.push({ path: '/tasks/new', query });
};

const onParentDialogSearch = (term: string) => {
  if (!task.value?.project_id) {
    setParentDialogOptions([]);
    return;
  }
  const extraFilters: Filter[] = [
    { column: 'id', operator: '<>', value: currentTaskId() },
    { column: 'project_id', operator: '=', value: task.value.project_id },
  ];
  void loadParentDialogTasks(term, extraFilters);
};

const openParentDialog = () => {
  parentDialogSelection.value = task.value?.parent_id ?? null;
  setParentDialogOptions([]);
  parentDialogOpen.value = true;
};

const persistParent = async (parentId: string | null) => {
  if (!task.value) return;
  showLoading();
  try {
    const updated = await TasksService.update(task.value.id, {
      project_id: task.value.project_id,
      parent_id: parentId,
      milestone_id: task.value.milestone_id,
      title: task.value.title,
      description: task.value.description,
      status: task.value.status,
      priority: task.value.priority,
      complexity: task.value.complexity,
      block_parent: task.value.block_parent,
      due_date: task.value.due_date,
      tag_ids: [...task.value.tag_ids],
    });
    task.value = updated;
    notify(
      parentId ? t('tasks.detail.action.parentAssigned') : t('tasks.detail.action.parentRemoved'),
      'positive',
    );
  } catch (err) {
    notifyError(t('common.error'), err);
  } finally {
    hideLoading();
  }
};

const confirmParentSelection = async () => {
  parentDialogOpen.value = false;
  await persistParent(parentDialogSelection.value);
};

const removeParent = async () => {
  await persistParent(null);
};

const pinCurrentTask = async () => {
  if (!task.value) return;
  try {
    const pinned: PinnedTask = {
      id: task.value.id,
      record_number: Number(task.value.record_number),
      title: task.value.title,
      project_name: task.value.project_name,
    };
    await userPreferencesStore.pinTask(pinned);
    notify(t('tasks.detail.action.pinSuccess'), 'positive');
  } catch (err) {
    notifyError(t('common.error'), err);
  }
};

const unpinCurrentTask = async () => {
  if (!task.value) return;
  try {
    await userPreferencesStore.unpinTask(task.value.id);
    notify(t('tasks.detail.action.unpinSuccess'), 'positive');
  } catch (err) {
    notifyError(t('common.error'), err);
  }
};

watch(
  () => route.params.id,
  async (newId) => {
    if (!newId) return;
    await loadTask(String(newId));
  },
  { immediate: true },
);
</script>
