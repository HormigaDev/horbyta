import { ref, onMounted, onUnmounted } from 'vue';
import type { WebSocketMessage, WebSocketConnectionState } from 'src/types/websocket';
import { useUI } from './useUI';

export interface UseWebSocketOptions {
  path?: string;
  autoConnect?: boolean;
  autoReconnect?: boolean;
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { path = '/ws', autoConnect = true, autoReconnect = true } = options;

  const { notify } = useUI();
  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const isConnecting = ref(false);
  const error = ref<string | null>(null);
  const state = ref<WebSocketConnectionState>('CLOSED');

  const getWebSocketUrl = (): string => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    return `${protocol}//${host}${path}`;
  };

  const connect = async (retries = 3): Promise<void> => {
    if (isConnected.value || isConnecting.value) return;

    isConnecting.value = true;
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
            state.value = 'OPEN';
            error.value = null;
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
    notify(`WebSocket connection failed: ${error.value}`, 'negative');

    if (lastError) {
      throw lastError;
    }
  };

  const disconnect = (): void => {
    if (ws.value) {
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
    if (!ws.value) {
      throw new Error('WebSocket is not initialized');
    }

    const messageHandler = (event: MessageEvent) => {
      try {
        const message = JSON.parse(event.data) as WebSocketMessage;
        if (message.type === type) {
          handler(message.data);
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.value.addEventListener('message', messageHandler);

    return () => {
      ws.value?.removeEventListener('message', messageHandler);
    };
  };

  const setupAutoReconnect = (maxRetries = 5) => {
    if (!ws.value) return;

    let reconnectAttempts = 0;

    const handleClose = () => {
      isConnected.value = false;
      state.value = 'RECONNECTING';

      if (reconnectAttempts < maxRetries && autoReconnect) {
        reconnectAttempts++;
        const delay = Math.pow(2, reconnectAttempts - 1) * 1000;
        setTimeout(() => {
          connect(1).catch((err) => {
            console.error('Auto-reconnect failed:', err);
          });
        }, delay);
      } else {
        state.value = 'CLOSED';
        error.value = 'Max reconnection attempts reached';
      }
    };

    ws.value.addEventListener('close', handleClose);
  };

  onMounted(async () => {
    if (!autoConnect) return;

    try {
      await connect();
      setupAutoReconnect();
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
