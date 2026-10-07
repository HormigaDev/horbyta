import { reactive, ref, computed } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { TasksService } from 'src/services/tasks';
import { TimeEntriesService } from 'src/services/timeEntries';
import { useUI } from 'src/composables/useUI';
import { useUiStateStore } from 'src/stores/uiState';
import type { Filter, QueryPayload } from 'src/types/query';
import type { KanbanTask } from 'src/types/kanban';
import type { TaskStatus } from 'src/types/task';

export interface KanbanBoardFilters {
  projectId: string | null;
  milestoneId: string | null;
  parentId: string | null;
  createdFrom: string | null;
  createdTo: string | null;
}

const BOARD_TASK_LIMIT = 100;
const BOARD_SCOPE = 'board' as const;
const BOARD_FILTERS_KEY = 'filters';

const toRfc3339Boundary = (date: string, boundary: 'start' | 'end') => {
  const time = boundary === 'start' ? '00:00:00.000Z' : '23:59:59.999Z';
  return `${date}T${time}`;
};

const isKanbanBoardFilters = (value: unknown): value is KanbanBoardFilters => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return false;
  }

  const candidate = value as Record<string, unknown>;
  const values = [
    candidate.projectId,
    candidate.milestoneId,
    candidate.parentId,
    candidate.createdFrom,
    candidate.createdTo,
  ];
  return values.every((item) => item === null || typeof item === 'string');
};

const defaultFilters = (): KanbanBoardFilters => ({
  projectId: null,
  milestoneId: null,
  parentId: null,
  createdFrom: null,
  createdTo: null,
});

export function useKanbanBoard() {
  const $q = useQuasar();
  const { t } = useI18n();
  const { notify, notifyError } = useUI();
  const uiState = useUiStateStore();

  const filters = reactive<KanbanBoardFilters>(defaultFilters());
  const tasks = ref<KanbanTask[]>([]);
  const total = ref(0);
  const isLoading = ref(false);
  const isMovingTask = ref(false);
  const moveAnnouncement = ref('');
  const appliedFilters = ref<KanbanBoardFilters>(defaultFilters());
  const activeFilterCount = computed(
    () => Object.values(appliedFilters.value).filter(Boolean).length,
  );
  const hasLoadedSavedFilters = ref(false);

  const applyFilters = (next: KanbanBoardFilters) => {
    filters.projectId = next.projectId;
    filters.milestoneId = next.milestoneId ?? null;
    filters.parentId = next.parentId ?? null;
    filters.createdFrom = next.createdFrom;
    filters.createdTo = next.createdTo;
  };

  const loadSavedUiFilters = async () => {
    if (hasLoadedSavedFilters.value) return;

    try {
      await uiState.ensureScopeLoaded(BOARD_SCOPE);
      const saved = uiState.getValue<KanbanBoardFilters | null>(
        BOARD_SCOPE,
        BOARD_FILTERS_KEY,
        null,
      );
      if (saved && isKanbanBoardFilters(saved)) {
        applyFilters(saved);
      }
    } catch (error) {
      console.error('Error loading remote board filters:', error);
    }

    hasLoadedSavedFilters.value = true;
  };

  const buildTaskFilters = (): Filter[] => {
    const result: Filter[] = [];

    if (filters.projectId) {
      result.push({ column: 'project_id', operator: '=', value: filters.projectId });
    }

    if (filters.milestoneId) {
      result.push({ column: 'milestone_id', operator: '=', value: filters.milestoneId });
    }

    if (filters.parentId) {
      result.push({ column: 'parent_id', operator: '=', value: filters.parentId });
    }

    if (filters.createdFrom) {
      result.push({
        column: 'created_at',
        operator: '>=',
        value: toRfc3339Boundary(filters.createdFrom, 'start'),
      });
    }

    if (filters.createdTo) {
      result.push({
        column: 'created_at',
        operator: '<=',
        value: toRfc3339Boundary(filters.createdTo, 'end'),
      });
    }

    return result;
  };

  const loadTasks = async () => {
    isLoading.value = true;
    try {
      await loadSavedUiFilters();
      try {
        await uiState.setValue<KanbanBoardFilters>(BOARD_SCOPE, BOARD_FILTERS_KEY, { ...filters });
      } catch {
        notify(t('ui.preferenceError'), 'warning');
      }

      const payload: QueryPayload = {
        page: 1,
        limit: BOARD_TASK_LIMIT,
        order_by: 'created_at',
        order_asc: false,
        filters: buildTaskFilters(),
      };

      const response = await TasksService.search(payload);
      appliedFilters.value = { ...filters };
      tasks.value = response.items;
      total.value = response.total;

      if (response.total > BOARD_TASK_LIMIT) {
        notify(t('board.limitExceeded'), 'info');
      }
    } catch (err: unknown) {
      notifyError(t('board.loadError'), err);
    } finally {
      isLoading.value = false;
    }
  };

  const taskHasKnownTimeEntries = (task: KanbanTask) => {
    if (typeof task.has_time_entries === 'boolean') return task.has_time_entries;
    if (typeof task.time_entry_count === 'number') return task.time_entry_count > 0;
    return null;
  };

  const taskHasTimeEntries = async (task: KanbanTask) => {
    const knownValue = taskHasKnownTimeEntries(task);
    if (knownValue !== null) return knownValue;

    const query: QueryPayload = {
      page: 1,
      limit: 1,
      order_by: 'created_at',
      order_asc: false,
      filters: [{ column: 'task_id', operator: '=', value: task.id }],
    };

    const response = await TimeEntriesService.search(query);
    return response.total > 0;
  };

  const validateTaskMove = async (task: KanbanTask, newColumnId: TaskStatus) => {
    if (newColumnId !== 'done') return true;

    const hasTimeEntries = await taskHasTimeEntries(task);
    if (hasTimeEntries) return true;

    $q.dialog({
      title: t('board.moveBlockedTitle'),
      message: t('board.moveBlockedMessage'),
      ok: { label: t('common.close'), color: 'primary', unelevated: true },
    });

    return false;
  };

  const updateTaskLocally = (taskId: string, status: TaskStatus) => {
    tasks.value = tasks.value.map((task) => (task.id === taskId ? { ...task, status } : task));
  };

  const moveTask = async (taskId: string, newColumnId: TaskStatus) => {
    const task = tasks.value.find((item) => item.id === taskId);
    if (!task || task.status === newColumnId || isMovingTask.value) return;

    const previousColumnId = task.status;
    isMovingTask.value = true;
    updateTaskLocally(taskId, newColumnId);

    try {
      const canMove = await validateTaskMove(task, newColumnId);
      if (!canMove) {
        updateTaskLocally(taskId, previousColumnId);
        return;
      }

      await TasksService.updateTaskStatus(taskId, newColumnId);
      moveAnnouncement.value = t('ui.moved', { status: t('tasks.form.status.' + newColumnId) });
      notify(moveAnnouncement.value, 'positive');
    } catch (err: unknown) {
      updateTaskLocally(taskId, previousColumnId);
      notifyError(t('board.moveError'), err);
    } finally {
      isMovingTask.value = false;
    }
  };

  const resetFilters = () => {
    filters.projectId = null;
    filters.milestoneId = null;
    filters.parentId = null;
    filters.createdFrom = null;
    filters.createdTo = null;
  };

  return {
    filters,
    tasks,
    total,
    isLoading,
    isMovingTask,
    activeFilterCount,
    moveAnnouncement,
    loadTasks,
    moveTask,
    resetFilters,
  };
}
