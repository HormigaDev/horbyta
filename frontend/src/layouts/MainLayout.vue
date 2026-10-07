<template>
  <q-layout view="hHh Lpr lFf">
    <q-header class="workspace-header">
      <q-toolbar>
        <q-btn
          flat
          dense
          round
          :icon="
            (!$q.screen.lt.md && drawerExpanded) || ($q.screen.lt.md && drawer)
              ? 'menu_open'
              : 'menu'
          "
          :aria-label="t('ui.navigation')"
          @click="toggleDrawer"
          class="q-mr-sm"
        />

        <q-img alt="Logo" src="/logo.png" width="36px" height="36px" class="q-mr-md" />
        <q-toolbar-title class="text-subtitle1 ellipsis" v-if="authStore.user">
          <span class="workspace-brand">Horbyta</span>
          <b class="text-weight-bold">{{ authStore.user.name }}</b>
        </q-toolbar-title>
        <q-toolbar-title class="text-weight-bold text-subtitle1 ellipsis" v-else>
          {{ t('layout.header.welcome') }}
        </q-toolbar-title>

        <q-space v-if="!$q.screen.lt.md" />

        <div v-if="!$q.screen.lt.md" class="row items-center">
          <task-search-component class="q-mr-md" />

          <q-btn
            flat
            round
            dense
            icon="notifications"
            :aria-label="t('layout.header.notifications')"
            class="q-mr-sm"
            @click="openNotificationsCenter"
          >
            <q-badge color="negative" floating v-if="notificationsStore.asyncUnreadCount > 0">
              {{ notificationsStore.asyncUnreadCount }}
            </q-badge>
            <q-tooltip>{{ t('layout.header.notifications') }}</q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="push_pin"
            :aria-label="t('layout.header.pinnedTasks')"
            class="q-mr-md"
            @click="pinnedTasksOpen = true"
          >
            <q-tooltip>{{ t('layout.header.pinnedTasks') }}</q-tooltip>
          </q-btn>

          <q-btn round flat :aria-label="t('ui.profile')">
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

                <q-item v-if="isAdmin" clickable @click="gotoAdminUsers">
                  <q-item-section avatar>
                    <q-icon name="admin_panel_settings" color="warning" />
                  </q-item-section>
                  <q-item-section>{{ t('layout.drawer.adminUsers') }}</q-item-section>
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
    <q-drawer
      v-model="drawer"
      show-if-above
      :mini="!$q.screen.lt.md && !drawerExpanded"
      :mini-width="60"
      :width="280"
      bordered
    >
      <template #mini>
        <div class="column flex-center q-pt-md q-gutter-y-md">
          <q-btn
            flat
            round
            dense
            icon="bar_chart"
            to="/"
            exact
            :color="getMiniRouteColor('/', true)"
            :aria-label="t('layout.drawer.dashboard')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.dashboard') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="view_kanban"
            to="/board"
            :color="getMiniRouteColor('/board')"
            :aria-label="t('layout.drawer.board')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ $t('layout.drawer.board') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="folder"
            to="/projects"
            :color="getMiniRouteColor('/projects')"
            :aria-label="t('layout.drawer.projects')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.projects') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="flag"
            to="/milestones"
            :color="getMiniRouteColor('/milestones')"
            :aria-label="t('layout.drawer.milestones')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.milestones') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="checklist"
            to="/tasks"
            :color="getMiniRouteColor('/tasks')"
            :aria-label="t('layout.drawer.tasks')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.tasks') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="compare_arrows"
            to="/change-requests"
            :color="getMiniRouteColor('/change-requests')"
            :aria-label="t('layout.drawer.changeRequests')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.changeRequests') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="people"
            to="/clients"
            :color="getMiniRouteColor('/clients')"
            :aria-label="t('layout.drawer.clients')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ $t('layout.drawer.clients') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="receipt_long"
            to="/invoices"
            :color="getMiniRouteColor('/invoices')"
            :aria-label="t('layout.drawer.invoices')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.invoices') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="paid"
            to="/payments"
            :color="getMiniRouteColor('/payments')"
            :aria-label="t('layout.drawer.payments')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.payments') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="settings"
            to="/settings"
            :color="getMiniRouteColor('/settings')"
            :aria-label="t('layout.drawer.settings')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.settings') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            v-if="isAdmin"
            flat
            round
            dense
            icon="admin_panel_settings"
            :color="getMiniRouteColor('/admin/users', true, 'warning')"
            to="/admin/users"
            :aria-label="t('layout.drawer.adminUsers')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.adminUsers') }}
            </q-tooltip>
          </q-btn>
        </div>

        <div class="absolute-bottom column flex-center q-pb-md q-gutter-y-sm w-100">
          <q-btn
            flat
            round
            dense
            icon="post_add"
            color="primary"
            to="/tasks/new"
            :aria-label="t('layout.drawer.newTask')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newTask') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="create_new_folder"
            color="primary"
            to="/projects/new"
            :aria-label="t('layout.drawer.newProject')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newProject') }}
            </q-tooltip>
          </q-btn>

          <q-btn
            flat
            round
            dense
            icon="flag_circle"
            color="primary"
            to="/milestones/new"
            :aria-label="t('layout.drawer.newMilestone')"
          >
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              {{ t('layout.drawer.newMilestone') }}
            </q-tooltip>
          </q-btn>
        </div>
      </template>

      <template v-if="!$q.screen.lt.md">
        <q-scroll-area class="fit">
          <q-list class="q-pt-sm">
            <div class="nav-group">{{ t('ui.work') }}</div>
            <q-item clickable v-ripple to="/" exact>
              <q-item-section avatar><q-icon name="bar_chart" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.dashboard') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/board">
              <q-item-section avatar><q-icon name="view_kanban" /></q-item-section>
              <q-item-section>{{ $t('layout.drawer.board') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/projects">
              <q-item-section avatar><q-icon name="folder" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.projects') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/milestones">
              <q-item-section avatar><q-icon name="flag" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.milestones') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/tasks">
              <q-item-section avatar><q-icon name="checklist" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.tasks') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/change-requests">
              <q-item-section avatar><q-icon name="compare_arrows" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.changeRequests') }}</q-item-section>
            </q-item>

            <div class="nav-group">{{ t('ui.customers') }}</div>
            <q-item clickable v-ripple to="/clients">
              <q-item-section avatar><q-icon name="people" /></q-item-section>
              <q-item-section>{{ $t('layout.drawer.clients') }}</q-item-section>
            </q-item>

            <div class="nav-group">{{ t('ui.finance') }}</div>
            <q-item clickable v-ripple to="/invoices">
              <q-item-section avatar><q-icon name="receipt_long" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.invoices') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/payments">
              <q-item-section avatar><q-icon name="paid" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.payments') }}</q-item-section>
            </q-item>

            <q-separator class="q-my-md" />

            <q-item clickable v-ripple to="/settings">
              <q-item-section avatar><q-icon name="settings" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.settings') }}</q-item-section>
            </q-item>

            <q-item v-if="isAdmin" clickable v-ripple to="/admin/users">
              <q-item-section avatar
                ><q-icon name="admin_panel_settings" color="warning"
              /></q-item-section>
              <q-item-section>{{ t('layout.drawer.adminUsers') }}</q-item-section>
            </q-item>

            <q-separator class="q-my-md" />

            <q-item clickable v-ripple class="text-primary" to="/tasks/new">
              <q-item-section avatar><q-icon name="post_add" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.newTask') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary" to="/projects/new">
              <q-item-section avatar><q-icon name="create_new_folder" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.newProject') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary" to="/milestones/new">
              <q-item-section avatar><q-icon name="flag_circle" /></q-item-section>
              <q-item-section>{{ t('layout.drawer.newMilestone') }}</q-item-section>
            </q-item>
          </q-list>

          <div
            class="text-caption text-center q-my-md"
            style="font-size: 10px; color: var(--text-muted)"
          >
            {{ appVersion }}
          </div>
        </q-scroll-area>
      </template>

      <!-- Mobile -->
      <template v-else>
        <q-scroll-area class="fit">
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
            <div class="q-px-md q-py-sm"><TaskSearchComponent class="full-width" /></div>
            <div class="nav-group">{{ t('ui.work') }}</div>
            <q-item clickable v-ripple to="/" exact>
              <q-item-section avatar>
                <q-icon name="bar_chart" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.dashboard') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/board">
              <q-item-section avatar>
                <q-icon name="view_kanban" />
              </q-item-section>
              <q-item-section>{{ $t('layout.drawer.board') }}</q-item-section>
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

            <div class="nav-group">{{ t('ui.finance') }}</div>
            <q-item clickable v-ripple to="/invoices">
              <q-item-section avatar>
                <q-icon name="receipt_long" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.invoices') }}</q-item-section>
            </q-item>

            <div class="nav-group">{{ t('ui.customers') }}</div>
            <q-item clickable v-ripple to="/clients">
              <q-item-section avatar>
                <q-icon name="people" />
              </q-item-section>
              <q-item-section>{{ $t('layout.drawer.clients') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/payments"
              ><q-item-section avatar><q-icon name="paid" /></q-item-section
              ><q-item-section>{{ t('layout.drawer.payments') }}</q-item-section></q-item
            >
            <q-separator class="q-my-md" />

            <q-item
              clickable
              v-ripple
              @click="
                openNotificationsCenter();
                drawer = false;
              "
            >
              <q-item-section avatar>
                <q-icon name="notifications" />
              </q-item-section>
              <q-item-section>{{ t('layout.header.notifications') }}</q-item-section>
              <q-item-section side v-if="notificationsStore.asyncUnreadCount > 0">
                <q-badge color="negative">{{ notificationsStore.asyncUnreadCount }}</q-badge>
              </q-item-section>
            </q-item>

            <q-item
              clickable
              v-ripple
              @click="
                pinnedTasksOpen = true;
                drawer = false;
              "
            >
              <q-item-section avatar><q-icon name="push_pin" /></q-item-section>
              <q-item-section>{{ t('layout.header.pinnedTasks') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/settings">
              <q-item-section avatar>
                <q-icon name="settings" />
              </q-item-section>
              <q-item-section>{{ t('layout.header.settings') }}</q-item-section>
            </q-item>

            <q-item v-if="isAdmin" clickable v-ripple to="/admin/users">
              <q-item-section avatar>
                <q-icon name="admin_panel_settings" color="warning" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.adminUsers') }}</q-item-section>
            </q-item>

            <q-separator class="q-my-md" />

            <q-item clickable v-ripple class="text-primary" to="/tasks/new">
              <q-item-section avatar>
                <q-icon name="post_add" />
              </q-item-section>
              <q-item-section>{{ t('layout.drawer.newTask') }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary" to="/projects/new">
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
          </q-list>

          <div
            class="text-caption text-center q-my-md"
            style="font-size: 10px; color: var(--text-muted)"
          >
            {{ appVersion }}
          </div>
        </q-scroll-area>
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
              @click="markAllNotificationsAsRead"
            />
            <q-btn
              flat
              round
              dense
              icon="close"
              :aria-label="t('common.close')"
              @click="notificationsCenterOpen = false"
            />
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

          <q-list v-else-if="userPreferencesStore.groupNotifications" class="notifications-list">
            <q-expansion-item
              v-for="group in asyncNotificationGroups"
              :key="group.key"
              switch-toggle-side
              expand-separator
              default-opened
              class="notification-group"
            >
              <template #header>
                <q-item-section>
                  <q-item-label class="text-weight-medium">{{ group.label }}</q-item-label>
                  <q-item-label caption>
                    {{
                      t('layout.notifications.groupCount', { count: group.notifications.length })
                    }}
                  </q-item-label>
                </q-item-section>
                <q-item-section side v-if="group.unreadCount > 0">
                  <q-badge color="negative">{{ group.unreadCount }}</q-badge>
                </q-item-section>
              </template>

              <q-item
                v-for="notif in group.notifications"
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
            </q-expansion-item>
          </q-list>

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
            @click="clearNotifications"
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

    <PinnedTasksDialog v-if="pinnedTasksOpen" v-model="pinnedTasksOpen" />

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
          <q-btn
            flat
            round
            dense
            icon="close"
            :aria-label="t('common.close')"
            @click="notificationDetailsOpen = false"
          />
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
import { computed, defineAsyncComponent, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useQuasar, type QNotifyCreateOptions } from 'quasar';
import { useAuthStore } from 'src/stores/auth';
import { APP_VERSION } from 'src/data/constants';
import { useNotificationsStore, type AsyncNotificationEvent } from 'src/stores/notifications';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { useUI } from 'src/composables/useUI';
import { useWebSocket } from 'src/composables/useWebSocket';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { AppNotification } from 'src/types/ui';
import { apiFetch } from 'src/boot/fetch';
import TaskSearchComponent from 'src/components/TaskSearchComponent.vue';

const PinnedTasksDialog = defineAsyncComponent(
  () => import('src/components/PinnedTasksDialog.vue'),
);

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const $q = useQuasar();
const { t } = useI18n();

const drawer = ref(!$q.screen.lt.md);
const drawerExpanded = ref(
  (() => {
    try {
      return localStorage.getItem('horbyta:drawer') === 'expanded';
    } catch {
      return false;
    }
  })(),
);
watch(drawerExpanded, (value) => {
  try {
    localStorage.setItem('horbyta:drawer', value ? 'expanded' : 'mini');
  } catch {
    /* Optional UI cache. */
  }
});
const pinnedTasksOpen = ref(false);
const notificationsCenterOpen = ref(false);
const notificationDetailsOpen = ref(false);
const selectedNotification = ref<AppNotification | null>(null);
const appVersion = APP_VERSION;
const notificationsStore = useNotificationsStore();
const userPreferencesStore = useUserPreferencesStore();
const { showErrorDialog } = useUI();
const websocket = useWebSocket();
const isAdmin = computed(() => authStore.user?.role === 'admin');

interface NotificationGroup {
  key: string;
  label: string;
  unreadCount: number;
  notifications: AppNotification[];
}

const asyncNotificationGroups = computed<NotificationGroup[]>(() => {
  const groups = new Map<string, AppNotification[]>();

  for (const notification of notificationsStore.asyncNotifications) {
    const key = notification.preferenceKey ?? notification.eventType ?? notification.type;
    const current = groups.get(key) ?? [];
    current.push(notification);
    groups.set(key, current);
  }

  return Array.from(groups.entries()).map(([key, notifications]) => ({
    key,
    label: resolveNotificationGroupLabel(key),
    unreadCount: notifications.filter((notification) => !notification.read).length,
    notifications,
  }));
});

websocket.on('notification', (data: unknown) => {
  if (!isAsyncNotificationEvent(data)) {
    return;
  }

  const event = localizeAsyncNotificationEvent(data);
  const notification = notificationsStore.addAsyncNotificationFromEvent(event);

  if (!event.silent) {
    const notifyOptions: QNotifyCreateOptions = {
      message: notification.title || notification.message,
      type: notification.type,
      position: 'top-right',
      timeout: 3500,
      actions: [{ icon: 'close', color: 'white', round: true, size: 'sm' }],
    };

    if (notification.body) {
      notifyOptions.caption = notification.body;
    }

    $q.notify(notifyOptions);
  }
});

websocket.on('notification.unread_count', (data: unknown) => {
  if (!isUnreadCountEvent(data)) {
    return;
  }

  notificationsStore.setAsyncUnreadCount(data.count);
});

async function logout() {
  await apiFetch('/auth/logout', { method: 'POST' }).catch(() => null);
  authStore.clearAuth();
  void router.push({ name: 'login' });
}

function gotoSettings() {
  void router.push({ name: 'settings' });
}

function gotoAdminUsers() {
  void router.push({ name: 'admin-users' });
}

function toggleDrawer() {
  if ($q.screen.lt.md) {
    drawer.value = !drawer.value;
    return;
  }

  drawer.value = true;
  drawerExpanded.value = !drawerExpanded.value;
}

function isActivePath(path: string, exact = false) {
  if (exact) {
    return route.path === path;
  }

  return route.path === path || route.path.startsWith(`${path}/`);
}

function getMiniRouteColor(path: string, exact = false, inactiveColor?: string) {
  return isActivePath(path, exact) ? 'primary' : inactiveColor;
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

function resolveNotificationGroupLabel(key: string) {
  const labels: Record<string, string> = {
    'client_portal.change_request_created': t('layout.notifications.group.changeRequests'),
    'client_portal.change_request_updated': t('layout.notifications.group.changeRequests'),
    'client_portal.change_request_approved': t('layout.notifications.group.changeRequests'),
    'client_portal.change_request_declined': t('layout.notifications.group.changeRequests'),
    'client_portal.change_request_cancelled': t('layout.notifications.group.changeRequests'),
    'billing.invoice_opened': t('layout.notifications.group.billing'),
    'billing.invoice_paid': t('layout.notifications.group.billing'),
    'billing.payment_received': t('layout.notifications.group.billing'),
    'work.milestone_released': t('layout.notifications.group.work'),
    positive: t('layout.notifications.group.positive'),
    negative: t('layout.notifications.group.negative'),
    warning: t('layout.notifications.group.warning'),
    info: t('layout.notifications.group.info'),
  };

  return labels[key] ?? key;
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
  markNotificationAsRead(notif);

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

  const route = getPayloadRoute(selectedNotification.value.payload);
  if (route) {
    notificationDetailsOpen.value = false;
    notificationsCenterOpen.value = false;
    void router.push(route);
    return;
  }

  showErrorDialog(selectedNotification.value.payload);
}

function markNotificationAsRead(notif: AppNotification) {
  if (!notif.backendId) {
    if (notif.unreadCount && notif.unreadCount > 1) {
      return;
    }

    notificationsStore.markAsRead(notif.id);
    return;
  }

  notificationsStore.markAsRead(notif.id);
  sendWebSocketMessage({
    type: 'notification.read',
    data: { id: notif.backendId },
  });
}

function markAllNotificationsAsRead() {
  notificationsStore.markAllAsRead();
  sendWebSocketMessage({ type: 'notification.read_all' });
}

function clearNotifications() {
  sendWebSocketMessage({ type: 'notification.clear_all' });
  notificationsStore.clearAll();
}

function sendWebSocketMessage(message: { type: string; data?: Record<string, unknown> }) {
  try {
    websocket.send(message);
  } catch (error) {
    console.debug('WebSocket notification acknowledgement skipped:', error);
  }
}

function localizeAsyncNotificationEvent(event: AsyncNotificationEvent): AsyncNotificationEvent {
  if (event.summaryType !== 'unread_notifications') {
    const translation = getNotificationTranslation(event.payload);
    if (translation) {
      const translatedEvent: AsyncNotificationEvent = {
        ...event,
        title: t(translation.titleKey, translation.params),
        message: t(translation.titleKey, translation.params),
      };

      if (translation.bodyKey) {
        translatedEvent.body = t(translation.bodyKey, translation.params);
      } else if (event.body !== undefined) {
        translatedEvent.body = event.body;
      }

      return translatedEvent;
    }

    return {
      ...event,
      title: event.title ?? event.message ?? t('layout.notifications.fallbackTitle'),
      message: event.message ?? event.title ?? t('layout.notifications.fallbackTitle'),
    };
  }

  const count = event.unreadCount ?? 0;

  return {
    ...event,
    title: t('layout.notifications.unreadSummaryTitle'),
    message: t('layout.notifications.unreadSummaryMessage', { count }),
    body: t('layout.notifications.unreadSummaryBody', { count }),
  };
}

function getNotificationTranslation(
  payload: unknown,
): { titleKey: string; bodyKey?: string; params: Record<string, unknown> } | null {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    return null;
  }

  const translation = (payload as Record<string, unknown>).translation;
  if (!translation || typeof translation !== 'object' || Array.isArray(translation)) {
    return null;
  }

  const translationRecord = translation as Record<string, unknown>;
  const titleKey = translationRecord.titleKey;
  const bodyKey = translationRecord.bodyKey;
  const params = translationRecord.params;

  if (typeof titleKey !== 'string' || titleKey.trim().length === 0) {
    return null;
  }

  const result: { titleKey: string; bodyKey?: string; params: Record<string, unknown> } = {
    titleKey,
    params:
      params && typeof params === 'object' && !Array.isArray(params)
        ? (params as Record<string, unknown>)
        : {},
  };

  if (typeof bodyKey === 'string' && bodyKey.trim().length > 0) {
    result.bodyKey = bodyKey;
  }

  return result;
}

function isAsyncNotificationEvent(data: unknown): data is AsyncNotificationEvent {
  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    return false;
  }

  const candidate = data as Record<string, unknown>;
  return (
    (candidate.id === undefined || typeof candidate.id === 'string') &&
    (candidate.message === undefined || typeof candidate.message === 'string') &&
    (candidate.title === undefined || typeof candidate.title === 'string') &&
    (candidate.body === undefined ||
      candidate.body === null ||
      typeof candidate.body === 'string') &&
    (candidate.summaryType === undefined || candidate.summaryType === 'unread_notifications') &&
    (candidate.silent === undefined || typeof candidate.silent === 'boolean') &&
    (candidate.read === undefined || typeof candidate.read === 'boolean') &&
    (candidate.unreadCount === undefined || typeof candidate.unreadCount === 'number')
  );
}

function isUnreadCountEvent(data: unknown): data is { count: number } {
  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    return false;
  }

  const count = (data as Record<string, unknown>).count;
  return typeof count === 'number' && Number.isFinite(count);
}

function getPayloadRoute(payload: unknown): string | null {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    return null;
  }

  const route = (payload as Record<string, unknown>).route;
  return typeof route === 'string' && route.trim().length > 0 ? route : null;
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

.notification-group {
  border: 1px solid var(--border-color);
  border-radius: 10px;
  overflow: hidden;
}

.notification-group .notifications-list-item {
  margin: 8px;
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

<style scoped>
.workspace-brand {
  font-weight: 700;
  color: var(--text-accent);
  letter-spacing: -0.3px;
  margin-right: 16px;
}
.workspace-header :deep(.q-toolbar__title b) {
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
}
@media (max-width: 599px) {
  .workspace-header :deep(.q-toolbar__title b) {
    display: none;
  }
}
</style>
