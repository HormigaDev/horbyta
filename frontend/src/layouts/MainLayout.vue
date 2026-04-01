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

          <q-btn flat round dense icon="notifications" class="q-mr-sm">
            <q-tooltip>Notificaciones</q-tooltip>
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

          <div class="text-caption text-grey text-center" style="font-size: 10px">
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
              <q-item-section avatar><q-icon name="notifications" /></q-item-section>
              <q-item-section>Notificaciones</q-item-section>
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

            <div class="text-caption text-grey text-center q-mt-md" style="font-size: 10px">
              {{ appVersion }}
            </div>
          </q-list>
        </div>
      </template>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from 'src/stores/auth';
import { useThemeStore } from 'src/stores/theme';
import { APP_VERSION } from 'src/data/constants';

const router = useRouter();
const authStore = useAuthStore();
const themeStore = useThemeStore();

const search = ref(''); // TODO: Implementar búsqueda
const drawer = ref(true);
const appVersion = APP_VERSION;

function logout() {
  authStore.clearAuth();
  void router.push({ name: 'login' });
}

function gotoSettings() {
  console.log('Ir a configuraciones... (TODO)');
}
</script>
