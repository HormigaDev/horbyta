import { useQuasar } from 'quasar';
import { isApiHttpError } from 'src/boot/fetch';
import { useI18n } from 'vue-i18n';
import { useNotificationsStore } from '../stores/notifications';
import ApiErrorDialog from '../components/ApiErrorDialog.vue';
import ErrorDetailsDialog from '../components/ErrorDetailsDialog.vue';
import type { NotificationType } from '../types/ui';
import { apiErrorKind, translateApiError, type ApiErrorTranslator } from '../utils/apiErrors';

interface ConfirmActionOptions {
  title: string;
  message: string;
  okLabel?: string;
  cancelLabel?: string;
  okColor?: string;
}

const toPayloadRecord = (value: unknown): Record<string, unknown> | null => {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  return null;
};

const STRUCTURED_ERROR_STATUSES = new Set<number>([400, 403, 404, 409, 422, 423]);

/** Código y datos del error, venga como ApiHttpError o ya serializado (notificaciones guardadas). */
const errorCodeOf = (payload: unknown): { code: string | null; params: string[] } => {
  if (isApiHttpError(payload)) return { code: payload.code, params: payload.params };
  const record = toPayloadRecord(payload);
  const code = typeof record?.code === 'string' ? record.code : null;
  const params = Array.isArray(record?.params) ? record.params.map((value) => String(value)) : [];
  return { code, params };
};

export function useUI() {
  const $q = useQuasar();
  const i18n = useI18n();
  const { t } = i18n;
  const notificationsStore = useNotificationsStore();
  const translator: ApiErrorTranslator = {
    t: (key) => t(key),
    has: (key) =>
      Object.prototype.hasOwnProperty.call(i18n.getLocaleMessage(i18n.locale.value), key),
  };

  const buildTechnicalPayload = (payload: unknown): unknown => {
    if (isApiHttpError(payload)) {
      return payload.toJSON();
    }

    if (payload instanceof Error) {
      return {
        name: payload.name,
        message: payload.message,
        stack: payload.stack,
      };
    }

    return payload;
  };

  /**
   * Mensaje para el usuario: la traducción del código del backend o, si no hay (error de red,
   * error interno), el mensaje de contexto que pasa quien llama.
   */
  const resolveErrorMessage = (fallbackMessage: string, payload?: unknown): string => {
    if (payload === undefined) return fallbackMessage;
    const { code, params } = errorCodeOf(payload);
    if (code === 'INTERNAL_ERROR') return fallbackMessage;
    return translateApiError(translator, code, params) ?? fallbackMessage;
  };

  const showApiErrorDialog = (
    status: number,
    message: string,
    translated: boolean,
    payload?: unknown,
  ) => {
    const kind = apiErrorKind(status);
    $q.dialog({
      component: ApiErrorDialog,
      componentProps: {
        kind,
        message,
        // Sin traducción específica se explica qué tipo de problema fue.
        hint: translated ? null : t(`error.apiDialog.hint.${kind}`),
        payload,
      },
    });
  };

  const notify = (message: string, type: NotificationType = 'info') => {
    notificationsStore.addNotification({ source: 'sync', message, type });
    $q.notify({
      message,
      type,
      position: 'top-right',
      timeout: 2500,
      actions: [
        { icon: 'close', 'aria-label': t('common.close'), color: 'white', round: true, size: 'sm' },
      ],
    });
  };

  const notifyAsync = (message: string, type: NotificationType = 'info', payload?: unknown) => {
    const actionList = [];
    if (payload) {
      actionList.push({
        label: t('common.details'),
        color: 'white',
        handler: () => showErrorDialog(payload),
      });
    }
    actionList.push({
      icon: 'close',
      'aria-label': t('common.close'),
      color: 'white',
      round: true,
      size: 'sm',
    });

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
    if (isApiHttpError(payload) && payload.status === 429) {
      notify(t('error.rateLimited'), 'warning');
      return;
    }

    if (isApiHttpError(payload) && STRUCTURED_ERROR_STATUSES.has(payload.status)) {
      const technicalPayload = buildTechnicalPayload(payload);
      const translated = translateApiError(translator, payload.code, payload.params);
      const summaryMessage = translated ?? message;

      showApiErrorDialog(payload.status, summaryMessage, translated !== null, technicalPayload);
      notificationsStore.addNotification({
        source: 'sync',
        message: summaryMessage,
        type: 'warning',
        payload: technicalPayload,
      });

      return;
    }

    const technicalPayload = payload === undefined ? undefined : buildTechnicalPayload(payload);
    const finalMessage = resolveErrorMessage(message, technicalPayload);

    if (isAsync) {
      notifyAsync(finalMessage, 'negative', technicalPayload);
    } else {
      const actionList = [];
      if (technicalPayload) {
        actionList.push({
          label: t('common.details'),
          color: 'white',
          handler: () => showErrorDialog(technicalPayload),
        });
      }
      actionList.push({
        icon: 'close',
        'aria-label': t('common.close'),
        color: 'white',
        round: true,
        size: 'sm',
      });

      notificationsStore.addNotification({
        source: 'sync',
        message: finalMessage,
        type: 'negative',
        payload: technicalPayload,
      });
      $q.notify({
        message: finalMessage,
        type: 'negative',
        position: 'top-right',
        timeout: 0,
        actions: actionList,
      });
    }
  };

  const confirmAction = ({
    title,
    message,
    okLabel = t('ui.confirm'),
    cancelLabel = t('common.cancel'),
    okColor = 'primary',
  }: ConfirmActionOptions) => {
    return new Promise<boolean>((resolve) => {
      $q.dialog({
        title,
        message,
        cancel: {
          label: cancelLabel,
          flat: true,
          color: 'grey',
        },
        ok: {
          label: okLabel,
          unelevated: true,
          color: okColor,
        },
        persistent: true,
      })
        .onOk(() => resolve(true))
        .onCancel(() => resolve(false));
    });
  };

  const confirmDelete = (itemName: string = t('ui.record')) => {
    return confirmAction({
      title: t('ui.deleteTitle'),
      message: t('ui.deleteMessage', { name: itemName }),
      okLabel: t('common.delete'),
      okColor: 'negative',
    });
  };

  const showLoading = (message: string = t('ui.processing')) => {
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
    resolveErrorMessage,
    notify,
    notifyAsync,
    notifyError,
    confirmAction,
    confirmDelete,
    showLoading,
    hideLoading,
    showErrorDialog,
  };
}
