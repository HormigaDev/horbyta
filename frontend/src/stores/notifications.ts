import { defineStore } from 'pinia';
import type { AppNotification } from '../types/ui';

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
