import { useQuasar } from 'quasar';
import { type ApiHttpError, isApiHttpError } from 'src/boot/fetch';
import { useI18n } from 'vue-i18n';
import { useNotificationsStore } from '../stores/notifications';
import BadRequestDialog from '../components/BadRequestDialog.vue';
import ErrorDetailsDialog from '../components/ErrorDetailsDialog.vue';
import type { NotificationType } from '../types/ui';

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

const splitCompositeMessage = (message: string): string[] => {
  return message
    .split(/\r?\n|;|\|/)
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
};

const collectMessagesFromUnknown = (value: unknown): string[] => {
  if (typeof value === 'string') {
    return splitCompositeMessage(value);
  }

  if (Array.isArray(value)) {
    return value.flatMap((item) => collectMessagesFromUnknown(item));
  }

  if (value && typeof value === 'object') {
    return Object.values(value as Record<string, unknown>).flatMap((item) =>
      collectMessagesFromUnknown(item),
    );
  }

  return [];
};

const uniqueMessages = (messages: string[]): string[] => {
  const seen = new Set<string>();
  const result: string[] = [];

  for (const message of messages) {
    const normalized = message.trim().toLowerCase();
    if (!normalized || seen.has(normalized)) {
      continue;
    }

    seen.add(normalized);
    result.push(message.trim());
  }

  return result;
};

interface BadRequestExtraction {
  messages: string[];
  hasUnmappedMessages: boolean;
}

export function useUI() {
  const $q = useQuasar();
  const { t } = useI18n();
  const notificationsStore = useNotificationsStore();

  const translateBackendMessage = (message: string): string | null => {
    const translationKey = `backendErrors.${message}`;

    const translated = t(translationKey);
    if (translated.startsWith('backendErrors')) {
      return null;
    }

    return translated;
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

  const resolveBackendError = (payload: unknown): string | null => {
    if (isApiHttpError(payload)) {
      if (payload.backendError && payload.backendError.trim().length > 0) {
        return payload.backendError.trim();
      }
      if (payload.message.trim().length > 0) {
        return payload.message.trim();
      }
      return null;
    }

    const payloadRecord = toPayloadRecord(payload);
    if (payloadRecord) {
      const maybeError = payloadRecord.error;
      if (typeof maybeError === 'string' && maybeError.trim().length > 0) {
        return maybeError.trim();
      }

      const maybeMessage = payloadRecord.message;
      if (typeof maybeMessage === 'string' && maybeMessage.trim().length > 0) {
        return maybeMessage.trim();
      }
    }

    if (payload instanceof Error && payload.message.trim().length > 0) {
      return payload.message.trim();
    }

    return null;
  };

  const resolveErrorMessage = (fallbackMessage: string, payload?: unknown): string => {
    if (payload === undefined) {
      return fallbackMessage;
    }

    const backendError = resolveBackendError(payload);
    if (!backendError) {
      return fallbackMessage;
    }

    const translated = translateBackendMessage(backendError);
    if (translated) {
      return translated;
    }

    return fallbackMessage;
  };

  const extractBadRequestMessages = (error: ApiHttpError): BadRequestExtraction => {
    const sourcePayload = toPayloadRecord(error.payload) ?? {};

    const candidateMessages = [
      error.backendError,
      sourcePayload.error,
      sourcePayload.errors,
      sourcePayload.validation_errors,
      sourcePayload.details,
    ].flatMap((entry) => collectMessagesFromUnknown(entry));

    const translated: string[] = [];
    let hasUnmappedMessages = false;

    for (const message of uniqueMessages(candidateMessages)) {
      const translatedMessage = translateBackendMessage(message);
      if (!translatedMessage) {
        hasUnmappedMessages = true;
        continue;
      }

      translated.push(translatedMessage);
    }

    const ordered = translated.sort((left, right) => left.localeCompare(right, 'es'));

    if (ordered.length === 0) {
      return {
        messages: [t('error.badRequestDialog.defaultMessage')],
        hasUnmappedMessages: true,
      };
    }

    return {
      messages: ordered,
      hasUnmappedMessages,
    };
  };

  const showBadRequestDialog = (summary: string, messages: string[]) => {
    $q.dialog({
      component: BadRequestDialog,
      componentProps: {
        summary,
        messages,
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
      actions: [{ icon: 'close', color: 'white', round: true, size: 'sm' }],
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
    if (isApiHttpError(payload) && payload.status === 400) {
      const technicalPayload = buildTechnicalPayload(payload);
      const summaryMessage = resolveErrorMessage(message, payload);
      const extraction = extractBadRequestMessages(payload);

      if (extraction.hasUnmappedMessages) {
        const genericMessage = t('error.badRequestDialog.defaultMessage');
        const actionList = [];
        if (technicalPayload) {
          actionList.push({
            label: t('common.details'),
            color: 'white',
            handler: () => showErrorDialog(technicalPayload),
          });
        }
        actionList.push({ icon: 'close', color: 'white', round: true, size: 'sm' });

        notificationsStore.addNotification({
          source: 'sync',
          message: genericMessage,
          type: 'warning',
          payload: technicalPayload,
        });
        $q.notify({
          message: genericMessage,
          type: 'warning',
          position: 'top-right',
          timeout: 3500,
          actions: actionList,
        });
        return;
      }

      showBadRequestDialog(summaryMessage, extraction.messages);
      notificationsStore.addNotification({
        source: 'sync',
        message: summaryMessage,
        type: 'warning',
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
      actionList.push({ icon: 'close', color: 'white', round: true, size: 'sm' });

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
        timeout: 2500,
        actions: actionList,
      });
    }
  };

  const confirmAction = ({
    title,
    message,
    okLabel = 'Confirmar',
    cancelLabel = 'Cancelar',
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

  const confirmDelete = (itemName: string = 'este registro') => {
    return confirmAction({
      title: 'Confirmar Eliminación',
      message: `¿Estás seguro de que deseas eliminar ${itemName}? Esta acción no se puede deshacer.`,
      okLabel: 'Eliminar',
      okColor: 'negative',
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
    confirmAction,
    confirmDelete,
    showLoading,
    hideLoading,
    showErrorDialog,
  };
}
