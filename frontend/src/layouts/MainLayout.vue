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

        <q-toolbar-title class="text-h6 ellipsis" v-if="authStore.user">
          Bienvenido <b class="text-weight-bold">{{ authStore.user.name }}</b>
        </q-toolbar-title>
        <q-toolbar-title class="text-weight-bold text-h6 ellipsis" v-else>
          Bienvenido
        </q-toolbar-title>

        <q-space v-if="!$q.screen.lt.md" />

        <div v-if="!$q.screen.lt.md" class="row items-center">
          <q-input
            v-model="search"
            dense
            outlined
            placeholder="Buscar tareas..."
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
            icon="smart_toy"
            class="q-mr-md"
            aria-label="Abrir asistente"
            @click="openChatbot"
          >
            <q-tooltip>Abrir asistente</q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="notifications" class="q-mr-sm">
            <q-badge color="negative" floating v-if="notificationsStore.asyncUnreadCount > 0">
              {{ notificationsStore.asyncUnreadCount }}
            </q-badge>
            <q-tooltip>Notificaciones</q-tooltip>
            <q-menu auto-close class="bg-dark">
              <q-list style="min-width: 300px; max-width: 350px">
                <q-item-label
                  header
                  class="row items-center justify-between text-white"
                  style="background-color: var(--q-dark-page)"
                >
                  Notificaciones
                  <q-btn
                    v-if="notificationsStore.asyncUnreadCount > 0"
                    flat
                    dense
                    size="sm"
                    color="primary"
                    label="Marcar leídas"
                    @click.stop="notificationsStore.markAllAsRead"
                  />
                </q-item-label>
                <q-separator />

                <template v-if="notificationsStore.asyncNotifications.length === 0">
                  <q-item>
                    <q-item-section class="text-center q-py-md" style="color: var(--text-muted)">
                      No hay notificaciones
                    </q-item-section>
                  </q-item>
                </template>

                <q-item
                  v-for="notif in notificationsStore.asyncNotifications.slice(0, 5)"
                  :key="notif.id"
                  clickable
                  :style="
                    !notif.read
                      ? 'border-bottom: 1px solid var(--border-color); background-color: var(--bg-hover)'
                      : 'border-bottom: 1px solid var(--border-color); background-color: var(--card-bg)'
                  "
                  @click="handleNotificationClick(notif)"
                >
                  <q-item-section avatar>
                    <q-icon
                      :name="
                        notif.type === 'positive'
                          ? 'check_circle'
                          : notif.type === 'negative'
                            ? 'error'
                            : notif.type === 'warning'
                              ? 'warning'
                              : 'info'
                      "
                      :color="notif.type"
                    />
                  </q-item-section>
                  <q-item-section>
                    <q-item-label
                      lines="2"
                      :class="!notif.read ? 'text-weight-bold' : ''"
                      style="color: var(--text-color)"
                      >{{ notif.message }}</q-item-label
                    >
                    <q-item-label caption style="color: var(--text-muted)">{{
                      new Date(notif.createdAt).toLocaleTimeString()
                    }}</q-item-label>
                  </q-item-section>
                  <q-item-section side v-if="!notif.read">
                    <q-badge rounded color="negative" />
                  </q-item-section>
                </q-item>

                <q-separator v-if="notificationsStore.asyncNotifications.length > 5" />
                <q-item
                  clickable
                  v-if="notificationsStore.asyncNotifications.length > 5"
                  class="text-center text-primary"
                  @click="notificationsStore.clearAll"
                >
                  <q-item-section>Borrar todas</q-item-section>
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>

          <q-btn
            flat
            round
            dense
            :icon="themeStore.isDark ? 'light_mode' : 'dark_mode'"
            @click="themeStore.toggleTheme()"
            class="q-mr-sm"
          >
            <q-tooltip>{{ themeStore.isDark ? 'Modo Claro' : 'Modo Oscuro' }}</q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="push_pin" class="q-mr-md">
            <q-tooltip>Tareas Fijadas</q-tooltip>
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
                      authStore.user?.name || 'Usuario'
                    }}</q-item-label>
                    <q-item-label caption>{{
                      authStore.user?.email || 'email@example.com'
                    }}</q-item-label>
                  </q-item-section>
                </q-item>

                <q-separator />

                <q-item clickable @click="gotoSettings">
                  <q-item-section avatar>
                    <q-icon name="settings" />
                  </q-item-section>
                  <q-item-section>Configuraciones</q-item-section>
                </q-item>

                <q-item clickable @click="logout" tag="a">
                  <q-item-section avatar>
                    <q-icon name="logout" color="negative" />
                  </q-item-section>
                  <q-item-section class="text-negative">Cerrar Sesión</q-item-section>
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>
        </div>
      </q-toolbar>
    </q-header>

    <q-drawer v-model="drawer" show-if-above :mini="!$q.screen.lt.md" :mini-width="60" bordered>
      <template v-if="!$q.screen.lt.md">
        <div class="column flex-center q-pt-md q-gutter-y-sm">
          <q-btn flat round dense icon="dashboard" to="/">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              Dashboard
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="folder" to="/projects">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              Proyectos
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="task" to="/tasks">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]"
              >Tareas</q-tooltip
            >
          </q-btn>

          <q-btn flat round dense icon="settings" to="/settings">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              Configuraciones
            </q-tooltip>
          </q-btn>
        </div>

        <div class="absolute-bottom column flex-center q-pb-md q-gutter-y-sm w-100">
          <q-btn flat round dense icon="post_add" color="primary">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              Nueva Tarea
            </q-tooltip>
          </q-btn>

          <q-btn flat round dense icon="create_new_folder" color="primary">
            <q-tooltip anchor="center right" self="center left" :offset="[10, 10]">
              Nuevo Proyecto
            </q-tooltip>
          </q-btn>

          <q-separator class="q-my-sm" inset />

          <div class="text-caption text-center" style="font-size: 10px; color: var(--text-muted)">
            {{ appVersion }}
          </div>
        </div>
      </template>

      <template v-else>
        <div class="column justify-between full-height">
          <div>
            <!-- Perfil del usuario -->
            <q-item class="q-py-md bg-transparent">
              <q-item-section avatar>
                <q-avatar size="48px" color="primary" text-color="white">
                  <q-icon name="person" />
                </q-avatar>
              </q-item-section>
              <q-item-section>
                <q-item-label class="text-weight-bold">{{
                  authStore.user?.name || 'Usuario'
                }}</q-item-label>
                <q-item-label caption>{{
                  authStore.user?.email || 'email@example.com'
                }}</q-item-label>
              </q-item-section>
            </q-item>

            <q-separator />

            <q-list>
              <q-item clickable v-ripple to="/">
                <q-item-section avatar>
                  <q-icon name="dashboard" />
                </q-item-section>
                <q-item-section>Dashboard</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/projects">
                <q-item-section avatar>
                  <q-icon name="folder" />
                </q-item-section>
                <q-item-section>Proyectos</q-item-section>
              </q-item>

              <q-item clickable v-ripple to="/tasks">
                <q-item-section avatar>
                  <q-icon name="task" />
                </q-item-section>
                <q-item-section>Tareas</q-item-section>
              </q-item>
            </q-list>
          </div>

          <q-list class="q-pb-md">
            <q-separator class="q-my-md" />

            <q-item clickable v-ripple>
              <q-item-section avatar>
                <q-icon name="notifications" />
              </q-item-section>
              <q-item-section>Notificaciones</q-item-section>
              <q-item-section side v-if="notificationsStore.asyncUnreadCount > 0">
                <q-badge color="negative">{{ notificationsStore.asyncUnreadCount }}</q-badge>
              </q-item-section>
            </q-item>

            <q-item clickable v-ripple @click="themeStore.toggleTheme()">
              <q-item-section avatar>
                <q-icon :name="themeStore.isDark ? 'light_mode' : 'dark_mode'" />
              </q-item-section>
              <q-item-section>{{
                themeStore.isDark ? 'Modo Claro' : 'Modo Oscuro'
              }}</q-item-section>
            </q-item>

            <q-item clickable v-ripple>
              <q-item-section avatar><q-icon name="push_pin" /></q-item-section>
              <q-item-section>Tareas Fijadas</q-item-section>
            </q-item>

            <q-item clickable v-ripple to="/settings">
              <q-item-section avatar>
                <q-icon name="settings" />
              </q-item-section>
              <q-item-section>Configuraciones</q-item-section>
            </q-item>

            <q-separator class="q-my-md" />

            <q-item clickable v-ripple class="text-primary">
              <q-item-section avatar>
                <q-icon name="post_add" />
              </q-item-section>
              <q-item-section>Nueva Tarea</q-item-section>
            </q-item>

            <q-item clickable v-ripple class="text-primary">
              <q-item-section avatar>
                <q-icon name="create_new_folder" />
              </q-item-section>
              <q-item-section>Nuevo Proyecto</q-item-section>
            </q-item>

            <q-item clickable v-ripple @click="logout" class="text-negative q-mt-md">
              <q-item-section avatar><q-icon name="logout" color="negative" /></q-item-section>
              <q-item-section>Cerrar Sesión</q-item-section>
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

    <ChatbotPanel v-if="chatbotPanelLoaded" v-model="chatbotOpen" />
  </q-layout>
</template>

<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from 'src/stores/auth';
import { useThemeStore } from 'src/stores/theme';
import { APP_VERSION } from 'src/data/constants';
import { useNotificationsStore } from 'src/stores/notifications';
import { useUI } from 'src/composables/useUI';
import type { AppNotification } from 'src/types/ui';

const router = useRouter();
const authStore = useAuthStore();
const themeStore = useThemeStore();

const search = ref(''); // TODO: Implementar búsqueda
const drawer = ref(true);
const chatbotOpen = ref(false);
const chatbotPanelLoaded = ref(false);
const appVersion = APP_VERSION;
const notificationsStore = useNotificationsStore();
const { showErrorDialog } = useUI();
const ChatbotPanel = defineAsyncComponent(() => import('src/components/ChatbotPanel.vue'));

onMounted(() => {
  notificationsStore.init();
});

function logout() {
  authStore.clearAuth();
  void router.push({ name: 'login' });
}

function gotoSettings() {
  console.log('Ir a configuraciones... (TODO)');
}

function openChatbot() {
  chatbotPanelLoaded.value = true;
  chatbotOpen.value = true;
}

function handleNotificationClick(notif: AppNotification) {
  notificationsStore.markAsRead(notif.id);
  if (notif.payload) {
    showErrorDialog(notif.payload);
  }
}
</script>
