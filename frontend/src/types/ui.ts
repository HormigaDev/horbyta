export type NotificationType = 'positive' | 'negative' | 'warning' | 'info';

export interface AppNotification {
  id: string;
  source: 'sync' | 'async';
  message: string;
  type: NotificationType;
  read: boolean;
  createdAt: string;
  payload?: unknown;
}
