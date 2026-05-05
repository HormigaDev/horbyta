import { defineStore } from 'pinia';
import type { AppNotification } from '../types/ui';

export interface AsyncNotificationEvent {
  id?: string;
  message?: string;
  title?: string;
  body?: string | null;
  type?: 'positive' | 'negative' | 'warning' | 'info';
  source?: 'sync' | 'async';
  payload?: unknown;
  createdAt?: string;
  preferenceKey?: string;
  eventType?: string;
  entityType?: string | null;
  entityId?: string | null;
  summaryType?: 'unread_notifications';
  unreadCount?: number;
}

export const useNotificationsStore = defineStore('notifications', {
  state: () => ({
    notifications: [] as AppNotification[],
    persistedAsyncUnreadCount: null as number | null,
  }),
  getters: {
    asyncUnreadCount: (state) => {
      const localUnreadCount = state.notifications
        .filter((n) => !n.read && n.source === 'async')
        .reduce((total, notification) => total + (notification.unreadCount ?? 1), 0);

      return state.persistedAsyncUnreadCount ?? localUnreadCount;
    },
    asyncNotifications: (state) => state.notifications.filter((n) => n.source === 'async'),
    allUnreadNotifications: (state) => state.notifications.filter((n) => !n.read),
    allNotifications: (state) => state.notifications,
  },
  actions: {
    addNotification(notif: Omit<AppNotification, 'id' | 'createdAt' | 'read'>) {
      const newNotif: AppNotification = {
        ...notif,
        id: crypto.randomUUID(),
        read: false,
        createdAt: new Date().toISOString(),
      };

      this.notifications.unshift(newNotif);
      return newNotif;
    },
    addAsyncNotificationFromEvent(event: AsyncNotificationEvent) {
      const backendId = typeof event.id === 'string' ? event.id : undefined;

      if (backendId) {
        const existing = this.notifications.find(
          (notification) => notification.backendId === backendId,
        );
        if (existing) {
          return existing;
        }
      }

      const title =
        typeof event.title === 'string' && event.title.trim().length > 0
          ? event.title.trim()
          : typeof event.message === 'string' && event.message.trim().length > 0
            ? event.message.trim()
            : 'Nueva notificación';

      const message =
        typeof event.message === 'string' && event.message.trim().length > 0
          ? event.message.trim()
          : title;

      const createdAt =
        typeof event.createdAt === 'string' && event.createdAt.trim().length > 0
          ? event.createdAt
          : new Date().toISOString();

      const newNotification: AppNotification = {
        id: crypto.randomUUID(),
        source: 'async',
        message,
        title,
        body: typeof event.body === 'string' ? event.body : null,
        type: event.type ?? 'info',
        read: false,
        createdAt,
        payload: event.payload,
      };
      if (typeof event.unreadCount === 'number' && Number.isFinite(event.unreadCount)) {
        newNotification.unreadCount = Math.max(0, Math.floor(event.unreadCount));
      }
      event.preferenceKey && (newNotification.preferenceKey = event.preferenceKey);
      event.eventType && (newNotification.eventType = event.eventType);
      event.entityType && (newNotification.entityType = event.entityType);
      event.entityId && (newNotification.entityId = event.entityId);
      event.summaryType && (newNotification.summaryType = event.summaryType);
      backendId && (newNotification.backendId = backendId);

      this.notifications.unshift(newNotification);
      if (this.persistedAsyncUnreadCount !== null) {
        this.persistedAsyncUnreadCount += newNotification.unreadCount ?? 1;
      }
      return newNotification;
    },
    markAsRead(id: string) {
      const target = this.notifications.find((n) => n.id === id);
      if (target) {
        target.read = true;
        if (target.source === 'async' && this.persistedAsyncUnreadCount !== null) {
          this.persistedAsyncUnreadCount = Math.max(
            0,
            this.persistedAsyncUnreadCount - (target.unreadCount ?? 1),
          );
        }
      }
    },
    markAllAsRead() {
      this.notifications.forEach((n) => {
        n.read = true;
      });
      this.persistedAsyncUnreadCount = 0;
    },
    setAsyncUnreadCount(count: number) {
      this.persistedAsyncUnreadCount = Math.max(0, Math.floor(count));
    },
    clearAll() {
      this.notifications = [];
    },
  },
});
