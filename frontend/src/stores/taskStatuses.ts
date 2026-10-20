import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { TaskStatusesService, type SaveTaskStatusDto } from 'src/services/taskStatuses';
import type { TaskStatus, TaskStatusFields, TaskStatusItem } from 'src/types/task';

// Nombres con los que la base crea los estados por defecto. Mientras no se renombren,
// se muestran traducidos; un nombre propio se muestra tal cual.
const DEFAULT_NAMES: Record<TaskStatus, string> = {
  todo: 'Todo',
  in_progress: 'In Progress',
  review: 'Review',
  done: 'Completed',
  cancelled: 'Cancelled',
};

export const TASK_STATUS_CATEGORIES: TaskStatus[] = [
  'todo',
  'in_progress',
  'review',
  'done',
  'cancelled',
];

export const useTaskStatusesStore = defineStore('taskStatuses', () => {
  const items = ref<TaskStatusItem[]>([]);
  const loaded = ref(false);
  let pending: Promise<void> | null = null;

  async function load(force = false) {
    if (loaded.value && !force) return;
    pending ??= TaskStatusesService.list()
      .then((result) => {
        items.value = result;
        loaded.value = true;
      })
      .finally(() => {
        pending = null;
      });
    await pending;
  }
  const byId = (id: string | null | undefined) => items.value.find((s) => s.id === id);
  const sorted = computed(() => [...items.value].sort((a, b) => a.position - b.position));
  /** Primer estado de una categoría (p. ej. el estado inicial de una tarea nueva). */
  const firstOf = (category: TaskStatus) => sorted.value.find((s) => s.category === category);

  async function create(data: SaveTaskStatusDto) {
    const created = await TaskStatusesService.create(data);
    items.value = [...items.value, created];
    return created;
  }
  async function update(id: string, data: SaveTaskStatusDto) {
    const updated = await TaskStatusesService.update(id, data);
    items.value = items.value.map((s) => (s.id === id ? updated : s));
    return updated;
  }
  async function remove(id: string) {
    await TaskStatusesService.delete(id);
    items.value = items.value.filter((s) => s.id !== id);
  }
  async function reorder(ids: string[]) {
    items.value = await TaskStatusesService.reorder(ids);
  }
  function reset() {
    items.value = [];
    loaded.value = false;
  }
  return { items, sorted, loaded, load, byId, firstOf, create, update, remove, reorder, reset };
});

type Translate = (key: string) => string;

/** Etiqueta visible de un estado (o de los datos de estado de una tarea). */
export function taskStatusLabel(
  t: Translate,
  status: Partial<TaskStatusItem> | Partial<TaskStatusFields> | null | undefined,
): string {
  if (!status) return '';
  const item = status as Partial<TaskStatusItem>;
  const fields = status as Partial<TaskStatusFields>;
  const key = item.key ?? fields.status_key ?? null;
  const name = item.name ?? fields.status_name ?? null;
  const category = item.category ?? fields.status;
  if (key && (!name || name === DEFAULT_NAMES[key])) return t(`tasks.form.status.${key}`);
  if (name) return name;
  return category ? t(`tasks.form.status.${category}`) : '';
}

export function useTaskStatusLabel() {
  const { t } = useI18n();
  return (status: Parameters<typeof taskStatusLabel>[1]) => taskStatusLabel(t, status);
}
