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
}

export const useNotificationsStore = defineStore('notifications', {
  state: () => ({
    notifications: [] as AppNotification[],
  }),
  getters: {
    asyncUnreadCount: (state) =>
      state.notifications.filter((n) => !n.read && n.source === 'async').length,
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
      event.preferenceKey && (newNotification.preferenceKey = event.preferenceKey);
      event.eventType && (newNotification.eventType = event.eventType);
      event.entityType && (newNotification.entityType = event.entityType);
      event.entityId && (newNotification.entityId = event.entityId);
      backendId && (newNotification.backendId = backendId);

      this.notifications.unshift(newNotification);
      return newNotification;
    },
    markAsRead(id: string) {
      const target = this.notifications.find((n) => n.id === id);
      if (target) {
        target.read = true;
      }
    },
    markAllAsRead() {
      this.notifications.forEach((n) => {
        n.read = true;
      });
    },
    clearAll() {
      this.notifications = [];
    },
  },
});
