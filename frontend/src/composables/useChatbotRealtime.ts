import { ref } from 'vue';

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

interface AsyncNotificationEvent {
  id?: string;
  message?: string;
  title?: string;
  body?: string;
  type?: 'positive' | 'negative' | 'warning' | 'info';
  source?: 'sync' | 'async';
  payload?: unknown;
  createdAt?: string;
  preferenceKey?: string;
}

interface WebSocketAdapter {
  isConnected: { value: boolean };
  connect: () => Promise<void>;
  disconnect: () => void;
  on: (type: string, handler: (data: unknown) => void) => () => void;
}

interface UseChatbotRealtimeOptions {
  websocket: WebSocketAdapter;
  t: (key: string) => string;
  notifyError: (message: string, payload?: unknown, isAsync?: boolean) => void;
  addAsyncNotificationFromEvent: (event: AsyncNotificationEvent) => void;
  onMessage: (content: string) => void;
  onGenerating: () => void;
  onTyping: (delta: string) => void;
  onError: () => void;
  onAiStatus: (payload: unknown) => void;
  onToolStatus: (payload: unknown) => void;
}

export function useChatbotRealtime(options: UseChatbotRealtimeOptions) {
  const wsUnsubscribers = ref<Array<() => void>>([]);
  const websocketBound = ref(false);

  const cleanupWebSocketListeners = () => {
    wsUnsubscribers.value.forEach((unsubscribe) => unsubscribe());
    wsUnsubscribers.value = [];
    websocketBound.value = false;
  };

  const bindWebSocketListeners = () => {
    if (websocketBound.value || !options.websocket.isConnected.value) {
      return;
    }

    const offMessage = options.websocket.on('message', (payload: unknown) => {
      const content = isRecord(payload) && typeof payload.content === 'string' ? payload.content : null;
      if (!content) {
        return;
      }

      options.onMessage(content);
    });

    const offGenerating = options.websocket.on('generating', () => {
      options.onGenerating();
    });

    const offTyping = options.websocket.on('typing', (payload: unknown) => {
      if (isRecord(payload) && typeof payload.delta === 'string') {
        options.onTyping(payload.delta);
      }
    });

    const offError = options.websocket.on('error', (payload: unknown) => {
      const message =
        isRecord(payload) && typeof payload.message === 'string'
          ? payload.message
          : options.t('chatbot.panel.sendError');

      options.onError();
      options.notifyError(message, payload, true);
    });

    const offNotification = options.websocket.on('notification', (payload: unknown) => {
      if (!isRecord(payload)) {
        return;
      }

      const notificationEvent: AsyncNotificationEvent = {
        source: 'async',
      };

      if (typeof payload.id === 'string') {
        notificationEvent.id = payload.id;
      }
      if (typeof payload.message === 'string') {
        notificationEvent.message = payload.message;
      }
      if (typeof payload.title === 'string') {
        notificationEvent.title = payload.title;
      }
      if (typeof payload.body === 'string') {
        notificationEvent.body = payload.body;
      }
      if (
        payload.type === 'positive' ||
        payload.type === 'negative' ||
        payload.type === 'warning' ||
        payload.type === 'info'
      ) {
        notificationEvent.type = payload.type;
      }
      if (payload.payload !== undefined) {
        notificationEvent.payload = payload.payload;
      }
      if (typeof payload.createdAt === 'string') {
        notificationEvent.createdAt = payload.createdAt;
      }
      if (typeof payload.preferenceKey === 'string') {
        notificationEvent.preferenceKey = payload.preferenceKey;
      }

      options.addAsyncNotificationFromEvent(notificationEvent);
    });

    const offAiStatus = options.websocket.on('ai_status', (payload: unknown) => {
      options.onAiStatus(payload);
    });

    const offToolStatus = options.websocket.on('tool_status', (payload: unknown) => {
      options.onToolStatus(payload);
    });

    wsUnsubscribers.value = [
      offMessage,
      offGenerating,
      offTyping,
      offError,
      offNotification,
      offAiStatus,
      offToolStatus,
    ];
    websocketBound.value = true;
  };

  const ensureWebSocketReady = async () => {
    if (!options.websocket.isConnected.value) {
      await options.websocket.connect();
    }

    bindWebSocketListeners();
  };

  const disconnectWebSocket = () => {
    cleanupWebSocketListeners();
    options.websocket.disconnect();
  };

  return {
    ensureWebSocketReady,
    disconnectWebSocket,
    cleanupWebSocketListeners,
    bindWebSocketListeners,
  };
}
