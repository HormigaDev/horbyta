<template>
  <router-view />
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { watch, onMounted } from 'vue';
import { useThemeStore } from 'src/stores/theme';
import { useAuthStore } from 'src/stores/auth';
import { useClientAuthStore } from 'src/stores/clientAuth';

const route = useRoute();
const themeStore = useThemeStore();
const authStore = useAuthStore();
const clientAuthStore = useClientAuthStore();

const isClientRoute = (path: string) => path.startsWith('/client') && !path.startsWith('/clients');

const applyRouteTheme = (routePath: string) => {
  if (isClientRoute(routePath)) {
    // Cliente siempre con tema claro y sin persistencia de preferencia.
    themeStore.applyTheme(false);
    return;
  }

  themeStore.applyTheme();
};

onMounted(() => {
  applyRouteTheme(route.path);
});

watch(
  () => route.fullPath,
  () => {
    if (
      route.name === 'error-not-found' &&
      !authStore.isAuthenticated &&
      !clientAuthStore.isAuthenticated
    ) {
      applyRouteTheme(route.path);
      return;
    }

    applyRouteTheme(route.path);
  },
);
</script>
