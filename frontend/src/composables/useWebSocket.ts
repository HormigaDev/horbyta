import { ref, onMounted, onUnmounted } from 'vue';
import type { WebSocketMessage, WebSocketConnectionState } from 'src/types/websocket';

export interface UseWebSocketOptions {
  path?: string;
  autoConnect?: boolean;
  autoReconnect?: boolean;
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { path = '/ws', autoConnect = true, autoReconnect = true } = options;

  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const isConnecting = ref(false);
  const error = ref<string | null>(null);
  const state = ref<WebSocketConnectionState>('CLOSED');
  const reconnectAttempts = ref(0);
  const manuallyClosed = ref(false);
  const listeners = ref<Map<string, Set<(data: unknown) => void>>>(new Map());

  const getWebSocketUrl = (): string => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}${path}`;
  };

  const dispatchMessage = (event: MessageEvent) => {
    try {
      const message = JSON.parse(event.data) as WebSocketMessage;
      const typedListeners = listeners.value.get(message.type);
      if (!typedListeners || typedListeners.size === 0) {
        return;
      }

      for (const handler of typedListeners) {
        handler(message.data);
      }
    } catch (err) {
      console.error('Failed to parse WebSocket message:', err);
    }
  };

  const reconnect = async (maxRetries = 5): Promise<void> => {
    if (!autoReconnect || manuallyClosed.value) {
      state.value = 'CLOSED';
      return;
    }

    if (reconnectAttempts.value >= maxRetries) {
      state.value = 'CLOSED';
      error.value = 'Max reconnection attempts reached';
      return;
    }

    reconnectAttempts.value += 1;
    state.value = 'RECONNECTING';
    const delay = Math.pow(2, reconnectAttempts.value - 1) * 1000;

    await new Promise((resolve) => setTimeout(resolve, delay));

    try {
      await connect(1);
      reconnectAttempts.value = 0;
    } catch (err) {
      console.error('Auto-reconnect failed:', err);
      await reconnect(maxRetries);
    }
  };

  const connect = async (retries = 3): Promise<void> => {
    if (isConnected.value || isConnecting.value) return;

    isConnecting.value = true;
    manuallyClosed.value = false;
    error.value = null;
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= retries; attempt++) {
      try {
        const url = getWebSocketUrl();
        ws.value = new WebSocket(url);

        await new Promise<void>((resolve, reject) => {
          if (!ws.value) {
            reject(new Error('WebSocket instance is null'));
            return;
          }

          const onOpen = () => {
            isConnected.value = true;
            isConnecting.value = false;
            state.value = 'OPEN';
            error.value = null;
            ws.value?.addEventListener('message', dispatchMessage);
            ws.value?.addEventListener('close', () => {
              isConnected.value = false;
              void reconnect();
            });
            cleanup();
            resolve();
          };

          const onError = (event: Event) => {
            lastError = new Error(`WebSocket error: ${event.type}`);
            reject(lastError);
          };

          const onClose = () => {
            reject(new Error('WebSocket closed unexpectedly'));
          };

          const cleanup = () => {
            ws.value?.removeEventListener('open', onOpen);
            ws.value?.removeEventListener('error', onError);
            ws.value?.removeEventListener('close', onClose);
          };

          ws.value.addEventListener('open', onOpen);
          ws.value.addEventListener('error', onError);
          ws.value.addEventListener('close', onClose);
        });

        return;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));
        if (attempt < retries) {
          const delay = Math.pow(2, attempt - 1) * 1000;
          await new Promise((resolve) => setTimeout(resolve, delay));
        }
      }
    }

    isConnecting.value = false;
    isConnected.value = false;
    state.value = 'CLOSED';
    error.value = lastError?.message ?? 'Failed to connect after retries';
    console.log(`WebSocket connection failed: ${error.value}`);

    if (lastError) {
      throw lastError;
    }
  };

  const disconnect = (): void => {
    manuallyClosed.value = true;
    if (ws.value) {
      ws.value.removeEventListener('message', dispatchMessage);
      ws.value.close();
      ws.value = null;
    }
    isConnected.value = false;
    state.value = 'CLOSED';
  };

  const send = (message: WebSocketMessage): void => {
    if (!isConnected.value || !ws.value) {
      throw new Error('WebSocket is not connected');
    }
    ws.value.send(JSON.stringify(message));
  };

  const on = (type: string, handler: (data: unknown) => void): (() => void) => {
    const currentListeners = listeners.value.get(type) ?? new Set<(data: unknown) => void>();
    currentListeners.add(handler);
    listeners.value.set(type, currentListeners);

    return () => {
      const typedListeners = listeners.value.get(type);
      if (!typedListeners) {
        return;
      }

      typedListeners.delete(handler);
      if (typedListeners.size === 0) {
        listeners.value.delete(type);
      }
    };
  };

  const setupAutoReconnect = () => {
    // Kept for compatibility with existing callers.
  };

  onMounted(async () => {
    if (!autoConnect) return;

    try {
      await connect();
    } catch (err) {
      console.error('Failed to connect WebSocket on mount:', err);
    }
  });

  onUnmounted(() => {
    disconnect();
  });

  return {
    isConnected,
    isConnecting,
    error,
    state,
    connect,
    disconnect,
    send,
    on,
    setupAutoReconnect,
  };
}
