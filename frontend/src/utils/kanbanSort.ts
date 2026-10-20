// Orden de las tarjetas dentro de cada columna del tablero.
import type { KanbanTask } from 'src/types/kanban';

export type KanbanSortField = 'due_date' | 'priority' | 'created_at' | 'updated_at';

export interface KanbanSort {
  by: KanbanSortField;
  /** Descendente: más alta, más reciente o más lejana primero. */
  desc: boolean;
}

export const KANBAN_SORT_FIELDS: KanbanSortField[] = [
  'due_date',
  'priority',
  'created_at',
  'updated_at',
];

// Orden con el que nace el tablero: lo que vence antes, arriba.
export const defaultKanbanSort = (): KanbanSort => ({ by: 'due_date', desc: false });

export const isKanbanSort = (value: unknown): value is KanbanSort => {
  if (!value || typeof value !== 'object') return false;
  const candidate = value as Record<string, unknown>;
  return (
    KANBAN_SORT_FIELDS.includes(candidate.by as KanbanSortField) &&
    typeof candidate.desc === 'boolean'
  );
};

const time = (value: string | null) => (value ? Date.parse(value) : null);

// Sin fecha límite va siempre al final, sea cual sea la dirección.
const compareDue = (a: KanbanTask, b: KanbanTask, desc: boolean) => {
  const left = time(a.due_date);
  const right = time(b.due_date);
  if (left === right) return 0;
  if (left === null) return 1;
  if (right === null) return -1;
  return desc ? right - left : left - right;
};

const primary = (a: KanbanTask, b: KanbanTask, sort: KanbanSort): number => {
  const direction = sort.desc ? -1 : 1;
  switch (sort.by) {
    case 'due_date':
      return compareDue(a, b, sort.desc);
    case 'priority':
      return direction * (a.priority - b.priority);
    case 'created_at':
      return direction * (Date.parse(a.created_at) - Date.parse(b.created_at));
    case 'updated_at':
      return direction * (Date.parse(a.updated_at) - Date.parse(b.updated_at));
  }
};

/**
 * Ordena las tareas; los empates se resuelven por fecha límite más próxima, luego por
 * prioridad más alta y, por último, por número de tarea (la más reciente primero).
 */
export const sortKanbanTasks = (tasks: KanbanTask[], sort: KanbanSort): KanbanTask[] =>
  [...tasks].sort(
    (a, b) =>
      primary(a, b, sort) ||
      compareDue(a, b, false) ||
      b.priority - a.priority ||
      Number(b.record_number) - Number(a.record_number),
  );
