export type WebSocketConnectionState = 'OPEN' | 'CLOSED' | 'RECONNECTING';

export interface WebSocketMessage {
  type: string;
  data?: unknown;
  id?: string;
}

export interface WebSocketError extends Error {
  code?: number;
}
