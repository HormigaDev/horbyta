export type NotificationType = 'positive' | 'negative' | 'warning' | 'info';

export interface AppNotification {
  id: string;
  source: 'sync' | 'async';
  message: string;
  title?: string;
  body?: string | null;
  type: NotificationType;
  read: boolean;
  createdAt: string;
  preferenceKey?: string;
  eventType?: string;
  entityType?: string;
  entityId?: string;
  backendId?: string;
  payload?: unknown;
}
