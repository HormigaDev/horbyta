<template>
  <q-layout view="lHh Lpr lFf">
    <q-header elevated>
      <q-toolbar>
        <q-toolbar-title> HormigaPM </q-toolbar-title>
        <q-btn
          flat
          round
          dense
          :icon="themeStore.isDark ? 'light_mode' : 'dark_mode'"
          @click="themeStore.toggleTheme()"
          :title="themeStore.isDark ? 'Modo Claro' : 'Modo Oscuro'"
        />
        <q-btn flat round dense icon="logout" @click="logout" title="Cerrar sesión" />
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from 'src/stores/auth';
import { useThemeStore } from 'src/stores/theme';

const router = useRouter();
const authStore = useAuthStore();
const themeStore = useThemeStore();

function logout() {
  authStore.clearAuth();
  router.push({ name: 'login' }).catch(() => null);
}
</script>
