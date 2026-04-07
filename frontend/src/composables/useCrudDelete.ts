import type { NotificationType } from '../types/ui';

interface UseCrudDeleteOptions {
  deleteById: (id: string) => Promise<unknown>;
  confirmDelete: (target: string) => Promise<boolean>;
  notify: (message: string, type?: NotificationType) => void;
  notifyError: (message: string, payload?: unknown) => void;
  buildDeleteTarget: (count: number) => string;
  buildDeleteSuccess: (count: number) => string;
  buildDeleteError: (count: number) => string;
  afterDelete?: () => void | Promise<void>;
}

export function useCrudDelete(options: UseCrudDeleteOptions) {
  const deleteSelected = async (selectedIds: string[]) => {
    if (selectedIds.length === 0) return;

    const confirmed = await options.confirmDelete(options.buildDeleteTarget(selectedIds.length));
    if (!confirmed) return;

    try {
      for (const id of selectedIds) {
        await options.deleteById(id);
      }

      options.notify(options.buildDeleteSuccess(selectedIds.length), 'positive');

      if (options.afterDelete) {
        await options.afterDelete();
      }
    } catch (err: unknown) {
      options.notifyError(options.buildDeleteError(selectedIds.length), err);
    }
  };

  return {
    deleteSelected,
  };
}
