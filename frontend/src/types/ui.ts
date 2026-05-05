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
  summaryType?: 'unread_notifications';
  backendId?: string;
  unreadCount?: number;
  payload?: unknown;
}
