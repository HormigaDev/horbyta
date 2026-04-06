import { defineStore } from 'pinia';
import type { AppNotification } from '../types/ui';

interface AsyncNotificationEvent {
  id?: string;
  message?: string;
  title?: string;
  body?: string | null;
  type?: 'positive' | 'negative' | 'warning' | 'info';
  source?: 'sync' | 'async';
  payload?: unknown;
  createdAt?: string;
  preferenceKey?: string;
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
    init() {
      const stored = localStorage.getItem('app_notifications');
      if (stored) {
        try {
          this.notifications = JSON.parse(stored) as AppNotification[];
        } catch {
          this.notifications = [];
        }
      }
    },
    save() {
      localStorage.setItem('app_notifications', JSON.stringify(this.notifications));
    },
    addNotification(notif: Omit<AppNotification, 'id' | 'createdAt' | 'read'>) {
      const newNotif: AppNotification = {
        ...notif,
        id: crypto.randomUUID(),
        read: false,
        createdAt: new Date().toISOString(),
      };

      this.notifications.unshift(newNotif);
      this.save();
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
        backendId,
        source: 'async',
        message,
        title,
        body: typeof event.body === 'string' ? event.body : null,
        type: event.type ?? 'info',
        read: false,
        createdAt,
        preferenceKey: event.preferenceKey,
        payload: event.payload,
      };

      this.notifications.unshift(newNotification);
      this.save();
      return newNotification;
    },
    markAsRead(id: string) {
      const target = this.notifications.find((n) => n.id === id);
      if (target) {
        target.read = true;
        this.save();
      }
    },
    markAllAsRead() {
      this.notifications.forEach((n) => {
        n.read = true;
      });
      this.save();
    },
    clearAll() {
      this.notifications = [];
      this.save();
    },
  },
});
