import { useQuasar } from 'quasar';
import { useNotificationsStore } from '../stores/notifications';
import ErrorDetailsDialog from '../components/ErrorDetailsDialog.vue';
import type { NotificationType } from '../types/ui';

export function useUI() {
  const $q = useQuasar();
  const notificationsStore = useNotificationsStore();

  const notify = (message: string, type: NotificationType = 'info') => {
    notificationsStore.addNotification({ source: 'sync', message, type });
    $q.notify({
      message,
      type,
      position: 'top-right',
      timeout: 2500,
      actions: [{ icon: 'close', color: 'white', round: true, size: 'sm' }],
    });
  };

  const notifyAsync = (message: string, type: NotificationType = 'info', payload?: unknown) => {
    const actionList = [];
    if (payload) {
      actionList.push({
        label: 'Ver detalles',
        color: 'white',
        handler: () => showErrorDialog(payload),
      });
    }
    actionList.push({ icon: 'close', color: 'white', round: true, size: 'sm' });

    $q.notify({
      message,
      type,
      position: 'top-right',
      timeout: 2500,
      actions: actionList,
    });

    notificationsStore.addNotification({ source: 'async', message, type, payload });
  };

  const notifyError = (message: string, payload?: unknown, isAsync = false) => {
    if (isAsync) {
      notifyAsync(message, 'negative', payload);
    } else {
      const actionList = [];
      if (payload) {
        actionList.push({
          label: 'Ver detalles',
          color: 'white',
          handler: () => showErrorDialog(payload),
        });
      }
      actionList.push({ icon: 'close', color: 'white', round: true, size: 'sm' });

      notificationsStore.addNotification({ source: 'sync', message, type: 'negative', payload });
      $q.notify({
        message,
        type: 'negative',
        position: 'top-right',
        timeout: 2500,
        actions: actionList,
      });
    }
  };

  const confirmDelete = (itemName: string = 'este registro') => {
    return new Promise<boolean>((resolve) => {
      $q.dialog({
        title: 'Confirmar Eliminación',
        message: `¿Estás seguro de que deseas eliminar ${itemName}? Esta acción no se puede deshacer.`,
        cancel: {
          label: 'Cancelar',
          flat: true,
          color: 'grey',
        },
        ok: {
          label: 'Eliminar',
          unelevated: true,
          color: 'negative',
        },
        persistent: true,
      })
        .onOk(() => resolve(true))
        .onCancel(() => resolve(false));
    });
  };

  const showLoading = (message: string = 'Procesando datos...') => {
    $q.loading.show({
      message,
      spinnerColor: 'primary',
    });
  };

  const hideLoading = () => $q.loading.hide();

  const showErrorDialog = (payload: unknown) => {
    $q.dialog({
      component: ErrorDetailsDialog,
      componentProps: { payload },
    });
  };

  return {
    notify,
    notifyAsync,
    notifyError,
    confirmDelete,
    showLoading,
    hideLoading,
    showErrorDialog,
  };
}
