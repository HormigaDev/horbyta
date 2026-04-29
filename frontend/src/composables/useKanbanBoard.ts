import { reactive, ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { TasksService } from 'src/services/tasks';
import { TimeEntriesService } from 'src/services/timeEntries';
import { useUI } from 'src/composables/useUI';
import type { Filter, QueryPayload } from 'src/types/query';
import type { KanbanTask } from 'src/types/kanban';
import type { TaskStatus } from 'src/types/task';

export interface KanbanBoardFilters {
  projectId: string | null;
  createdFrom: string | null;
  createdTo: string | null;
}

const BOARD_TASK_LIMIT = 100;
const BOARD_FILTERS_STORAGE_KEY = 'kanban_board_filters';

const toRfc3339Boundary = (date: string, boundary: 'start' | 'end') => {
  const time = boundary === 'start' ? '00:00:00.000Z' : '23:59:59.999Z';
  return `${date}T${time}`;
};

const isKanbanBoardFilters = (value: unknown): value is KanbanBoardFilters => {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return false;
  }

  const candidate = value as Record<string, unknown>;
  const values = [candidate.projectId, candidate.createdFrom, candidate.createdTo];
  return values.every((item) => item === null || typeof item === 'string');
};

const loadSavedFilters = (): KanbanBoardFilters => {
  const fallback: KanbanBoardFilters = {
    projectId: null,
    createdFrom: null,
    createdTo: null,
  };

  try {
    const raw = localStorage.getItem(BOARD_FILTERS_STORAGE_KEY);
    if (!raw) return fallback;

    const parsed: unknown = JSON.parse(raw);
    return isKanbanBoardFilters(parsed) ? parsed : fallback;
  } catch {
    return fallback;
  }
};

const saveFilters = (filters: KanbanBoardFilters) => {
  localStorage.setItem(BOARD_FILTERS_STORAGE_KEY, JSON.stringify(filters));
};

export function useKanbanBoard() {
  const $q = useQuasar();
  const { t } = useI18n();
  const { notify, notifyError } = useUI();

  const filters = reactive<KanbanBoardFilters>(loadSavedFilters());
  const tasks = ref<KanbanTask[]>([]);
  const total = ref(0);
  const isLoading = ref(false);
  const isMovingTask = ref(false);

  const buildTaskFilters = (): Filter[] => {
    const result: Filter[] = [];

    if (filters.projectId) {
      result.push({ column: 'project_id', operator: '=', value: filters.projectId });
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
      saveFilters(filters);

      const payload: QueryPayload = {
        page: 1,
        limit: BOARD_TASK_LIMIT,
        order_by: 'created_at',
        order_asc: false,
        filters: buildTaskFilters(),
      };

      const response = await TasksService.search(payload);
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
    } catch (err: unknown) {
      updateTaskLocally(taskId, previousColumnId);
      notifyError(t('board.moveError'), err);
    } finally {
      isMovingTask.value = false;
    }
  };

  const resetFilters = () => {
    filters.projectId = null;
    filters.createdFrom = null;
    filters.createdTo = null;
  };

  return {
    filters,
    tasks,
    total,
    isLoading,
    isMovingTask,
    loadTasks,
    moveTask,
    resetFilters,
  };
}
