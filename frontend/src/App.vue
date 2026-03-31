<template>
  <router-view />
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { watch, onMounted } from 'vue';
import { useThemeStore } from 'src/stores/theme';
import { useAuthStore } from 'src/stores/auth';
import { apiFetch } from 'src/boot/fetch';
import { useRouter } from 'vue-router';

const route = useRoute();
const router = useRouter();
const themeStore = useThemeStore();
const authStore = useAuthStore();

onMounted(async () => {
  themeStore.applyTheme(route.name === 'login' ? true : undefined);

  try {
    await apiFetch('/health');
    authStore.setAuthenticated(true);
    if (route.name === 'login') {
      void router.push({ name: 'dashboard' });
    }
  } catch {
    authStore.clearAuth();
    if (route.meta.requiresAuth) {
      void router.push({ name: 'login' });
    }
  }
});

watch(
  () => route.name,
  (newRouteName) => {
    if (newRouteName === 'login') {
      themeStore.applyTheme(true);
    } else if (newRouteName === 'error-not-found' && !authStore.isAuthenticated) {
      themeStore.applyTheme(true);
    } else {
      themeStore.applyTheme();
    }
  },
);
</script>
