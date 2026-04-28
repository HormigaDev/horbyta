<template>
  <q-layout view="hHh Lpr lFf">
    <q-header class="q-py-xs">
      <q-toolbar>
        <q-btn
          v-if="$q.screen.lt.md"
          flat
          dense
          round
          icon="menu"
          aria-label="Menu"
          @click="drawer = !drawer"
          class="q-mr-sm"
        />

        <q-img alt="Logo" src="/logo.png" width="36px" height="36px" class="q-mr-md q-ml-md" />
        <q-toolbar-title class="text-subtitle1 ellipsis" v-if="authStore.user">
          {{ t('layout.header.welcome') }} <b class="text-weight-bold">{{ authStore.user.name }}</b>
        </q-toolbar-title>
        <q-toolbar-title class="text-weight-bold text-subtitle1 ellipsis" v-else>
          {{ t('layout.header.welcome') }}
        </q-toolbar-title>

        <q-space v-if="!$q.screen.lt.md" />

        <div v-if="!$q.screen.lt.md" class="row items-center">
          <q-input
            v-model="search"
            dense
            outlined
            :placeholder="t('layout.header.searchPlaceholder')"
            class="q-mr-md"
            style="width: 250px"
          >
            <template v-slot:append>
              <q-icon name="search" />
            </template>
          </q-input>

          <q-btn
            flat
            round
            dense
            icon="notifications"
            class="q-mr-sm"
            @click="openNotificationsCenter"
          >
            <q-badge color="negative" floating v-if="notificationsStore.asyncUnreadCount > 0">
              {{ notificationsStore.asyncUnreadCount }}
            </q-badge>
            <q-tooltip>{{ t('layout.header.notifications') }}</q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="push_pin" class="q-mr-md">
            <q-tooltip>{{ t('layout.header.pinnedTasks') }}</q-tooltip>
          </q-btn>

          <q-btn round flat>
            <q-avatar size="32px" color="primary" text-color="white">
              <q-icon name="person" />
            </q-avatar>
            <q-menu auto-close>
              <q-list style="min-width: 200px">
                <q-item>
                  <q-item-section>
                    <q-item-label class="text-weight-bold">{{
                      authStore.user?.name || t('layout.profile.defaultUser')
                    }}</q-item-label>
                    <q-item-label caption>{{
                      authStore.user?.email || t('layout.profile.defaultEmail')
                    }}</q-item-label>
                  </q-item-section>
                </q-item>

                <q-separator />

                <q-item clickable @click="gotoSettings">
                  <q-item-section avatar>
                    <q-icon name="settings" />
                  </q-item-section>
                  <q-item-section>{{ t('layout.header.settings') }}</q-item-section>
                </q-item>

                <q-item clickable @click="logout" tag="a">
                  <q-item-section avatar>
                    <q-icon name="logout" color="negative" />
                  </q-item-section>
                  <q-item-section class="text-negative">{{
                    t('layout.header.logout')
                  }}</q-item-section>
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>
        </div>
      </q-toolbar>
    </q-header>

    <!-- PC -->
    <q-drawer v-model="drawer" show-if-above :mini="!$q.screen.lt.md" :mini-width="60" bordered>
      <template v-if="!$q.screen.lt.md">
        <div class="column flex-center q-pt-md q-gutter-y-md">
          <q-btn flat round dense icon="bar_chart" to="/">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.dashboard') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="folder" to="/projects">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.projects') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="flag" to="/milestones">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.milestones') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="checklist" to="/tasks">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">{{
              t('layout.drawer.tasks')
            }}</q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="compare_arrows" to="/change-requests">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.changeRequests') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="people" to="/clients">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ $t('layout.drawer.clients') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="receipt_long" to="/invoices">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.invoices') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="paid" to="/payments">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.payments') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="settings" to="/settings">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.settings') }}
            </q-tooltip>
          </q-btn>
        </div>

        <div class="absolute-bottom column flex-center q-pb-md q-gutter-y-sm w-100">
          <q-btn flat round dense icon="post_add" color="primary" to="/tasks/new">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newTask') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="create_new_folder" color="primary" to="/projects/new">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newProject') }}
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="flag_circle" color="primary" to="/milestones/new">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newMilestone') }}
            </q-tooltip>
          </q-btn>

          <q-separator class="q-my-sm" inset />

          <div class="text-caption text-center" style="font-size: 10px; color: var(--text-muted)">
            {{ appVersion }}
          </div>
        </div>
      </template>

      <!-- Mobile -->
      <template v-else>
        <div class="column justify-between full-height">
          <div>
            <q-item class="q-py-md bg-transparent">
              <q-item-section avatar>
                <q-avatar size="48px" color="primary" text-color="white">
                  <q-icon name="person" />
                </q-avatar>
              </q-item-section>
              <q-item-section>
                <q-item-label class="text-weight-bold">{{
                  authStore.user?.name || t('layout.profile.defaultUser')
                }}</q-item-label>
                <q-item-label caption>{{
                  authStore.user?.email || t('layout.profile.defaultEmail')
                }}</q-item-label>
              </q-item-section>
            </q-item>

            <q-separator />

            <q-list>
              <q-item clickable v-ripple to="/">
                <q-item-section avatar>
                  <q-icon name="bar_chart" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.dashboard') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/projects">
                <q-item-section avatar>
                  <q-icon name="folder" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.projects') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/milestones">
                <q-item-section avatar>
                  <q-icon name="flag" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.milestones') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/tasks">
                <q-item-section avatar>
                  <q-icon name="checklist" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.tasks') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/change-requests">
                <q-item-section avatar>
                  <q-icon name="compare_arrows" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.changeRequests') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/invoices">
                <q-item-section avatar>
                  <q-icon name="receipt_long" />
                </q-item-section>
                <q-item-section>{{ t('layout.drawer.invoices') }}</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/clients">
                <q-item-section avatar>
                  <q-icon name="people" />
                </q-item-section>
                <q-item-section>{{ $t('layout.drawer.clients') }}</q-item-section>
              </q-item>
            </q-list>
          </div>

          <q-list class="q-pb-md">
            <q-separator class="q-my-md" />

            <q-item clickable v-ripple>
              <q-item-section avatar>
                <q-icon name="notifications" />
              </q-item-section>
              <q-item-section>{{ t('layout.header.notifications') }}</q-item-section>
              <q-item-section side v-if="notificationsStore.asyncUnreadCount > 0">
                <q-badge color="negative">{{ notificationsStore.asyncUnreadCount }}</q-badge>
              </q-item-section>
            </q-item>

            <q-item clickable v-ripple>
              <q-item-section avatar><q-icon name="push_pin" /></q-item-section>
              <q-item-section>{{ t('layout.header.pinnedTasks') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/settings">
              <q-item-section avatar>
                <q-icon name="settings" />
              </q-item-section>
              <q-item-section>{{ t('layout.header.settings') }}</q-item-section>
            </q-item>

            <q-separator class="q-my-md" />

            <q-item clickable v-ripple class="text-primary">
              <q-item-section avatar>
                <q-icon name="post_add" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.newTask') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary">
              <q-item-section avatar>
                <q-icon name="create_new_folder" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.newProject') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary" to="/milestones/new">
              <q-item-section avatar>
                <q-icon name="flag_circle" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.newMilestone') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple @click="logout" class="text-negative q-mt-md">
              <q-item-section avatar><q-icon name="logout" color="negative" /></q-item-section>
              <q-item-section>{{ t('layout.header.logout') }}</q-item-section>
            </q-item>

            <div
              class="text-caption text-center q-mt-md"
              style="font-size: 10px; color: var(--text-muted)"
            >
              {{ appVersion }}
            </div>
          </q-list>
        </div>
      </template>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>

    <q-dialog v-model="notificationsCenterOpen" maximized persistent>
      <q-card class="notifications-center-card">
        <q-card-section class="row items-center justify-between notifications-center-header">
          <div>
            <div class="text-h6">{{ t('layout.header.notifications') }}</div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('layout.notifications.subtitle') }}
            </div>
          </div>

          <div class="row items-center q-gutter-sm">
            <q-btn
              v-if="notificationsStore.asyncUnreadCount > 0"
              flat
              dense
              color="primary"
              :label="t('layout.header.markAllRead')"
              @click="notificationsStore.markAllAsRead"
            />
            <q-btn flat round dense icon="close" @click="notificationsCenterOpen = false" />
          </div>
        </q-card-section>

        <q-separator />

        <q-card-section class="notifications-center-body">
          <div
            v-if="notificationsStore.asyncNotifications.length === 0"
            class="notifications-empty-state"
          >
            <q-icon name="notifications_none" size="40px" color="grey-6" />
            <div class="text-subtitle1 q-mt-sm">{{ t('layout.header.noNotifications') }}</div>
          </div>

          <q-list v-else class="notifications-list">
            <q-item
              v-for="notif in notificationsStore.asyncNotifications"
              :key="notif.id"
              clickable
              class="notifications-list-item"
              :class="{ 'notifications-list-item-unread': !notif.read }"
              @click="handleNotificationClick(notif)"
            >
              <q-item-section avatar>
                <q-avatar :color="resolveNotificationColor(notif.type)" text-color="white">
                  <q-icon :name="resolveNotificationIcon(notif.type)" />
                </q-avatar>
              </q-item-section>

              <q-item-section>
                <q-item-label class="text-weight-medium">{{
                  notif.title || notif.message
                }}</q-item-label>
                <q-item-label lines="2" caption>
                  {{ notif.body || notif.message }}
                </q-item-label>
                <q-item-label caption class="q-mt-xs">{{
                  formatNotificationDate(notif.createdAt)
                }}</q-item-label>
              </q-item-section>

              <q-item-section side top>
                <q-badge v-if="!notif.read" rounded color="negative" class="q-mt-sm" />
              </q-item-section>
            </q-item>
          </q-list>
        </q-card-section>

        <q-separator />

        <q-card-actions align="between" class="q-px-md q-py-sm">
          <q-btn
            flat
            color="negative"
            :label="t('layout.header.clearAll')"
            @click="notificationsStore.clearAll"
          />
          <q-btn
            flat
            color="primary"
            :label="t('common.close')"
            @click="notificationsCenterOpen = false"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="notificationDetailsOpen">
      <q-card class="notification-detail-card">
        <q-card-section class="row items-start justify-between">
          <div>
            <div class="text-h6">
              {{ selectedNotification?.title || selectedNotification?.message }}
            </div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{
                selectedNotification ? formatNotificationDate(selectedNotification.createdAt) : ''
              }}
            </div>
          </div>
          <q-btn flat round dense icon="close" @click="notificationDetailsOpen = false" />
        </q-card-section>

        <q-separator />

        <q-card-section>
          <div class="notification-detail-body">
            {{ selectedNotification?.body || selectedNotification?.message }}
          </div>
        </q-card-section>

        <q-separator />

        <q-card-actions align="right">
          <q-btn
            v-if="selectedNotification?.payload"
            flat
            color="primary"
            :label="t('common.details')"
            @click="openSelectedNotificationPayload"
          />
          <q-btn
            flat
            color="primary"
            :label="t('common.close')"
            @click="notificationDetailsOpen = false"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-layout>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from 'src/stores/auth';
import { APP_VERSION } from 'src/data/constants';
import { useNotificationsStore } from 'src/stores/notifications';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useUI } from 'src/composables/useUI';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { AppNotification } from 'src/types/ui';
import { apiFetch } from 'src/boot/fetch';

const router = useRouter();
const authStore = useAuthStore();
const { t } = useI18n();

const search = ref(''); // TODO: Implementar búsqueda
const drawer = ref(true);
const notificationsCenterOpen = ref(false);
const notificationDetailsOpen = ref(false);
const selectedNotification = ref<AppNotification | null>(null);
const appVersion = APP_VERSION;
const notificationsStore = useNotificationsStore();
const userPreferencesStore = useUserPreferencesStore();
const { showErrorDialog } = useUI();

onMounted(() => {
  notificationsStore.init();
});

async function logout() {
  await apiFetch('/auth/logout', { method: 'POST' }).catch(() => null);
  authStore.clearAuth();
  void router.push({ name: 'login' });
}

function gotoSettings() {
  void router.push({ name: 'settings' });
}

function openNotificationsCenter() {
  notificationsCenterOpen.value = true;
}

function resolveNotificationIcon(type: AppNotification['type']) {
  if (type === 'positive') return 'check_circle';
  if (type === 'negative') return 'error';
  if (type === 'warning') return 'warning';
  return 'info';
}

function resolveNotificationColor(type: AppNotification['type']) {
  if (type === 'positive') return 'positive';
  if (type === 'negative') return 'negative';
  if (type === 'warning') return 'warning';
  return 'info';
}

function formatNotificationDate(rawDate: string) {
  return formatDateByPreferences(rawDate, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback: t('common.empty'),
  });
}

function handleNotificationClick(notif: AppNotification) {
  notificationsStore.markAsRead(notif.id);

  if (notif.body && notif.body.trim().length > 0) {
    selectedNotification.value = notif;
    notificationDetailsOpen.value = true;
    return;
  }

  if (notif.payload) {
    showErrorDialog(notif.payload);
  }
}

function openSelectedNotificationPayload() {
  if (!selectedNotification.value?.payload) {
    return;
  }

  showErrorDialog(selectedNotification.value.payload);
}
</script>

<style scoped>
.notifications-center-card {
  width: min(860px, 96vw);
  max-width: 96vw;
  height: min(82vh, 780px);
  margin: auto;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--card-bg);
  display: flex;
  flex-direction: column;
}

.notifications-center-header {
  flex-shrink: 0;
}

.notifications-center-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.notifications-empty-state {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  color: var(--text-muted);
}

.notifications-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.notifications-list-item {
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 8px;
}

.notifications-list-item-unread {
  background: var(--bg-hover);
}

.notification-detail-card {
  width: min(680px, 94vw);
  max-width: 94vw;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
}

.notification-detail-body {
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-color);
}

@media (max-width: 768px) {
  .notifications-center-card {
    width: 100vw;
    height: 100dvh;
    border-radius: 0;
    margin: 0;
  }
}
</style>
