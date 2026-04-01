<template>
  <router-view />
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { watch, onMounted } from 'vue';
import { useThemeStore } from 'src/stores/theme';
import { useAuthStore } from 'src/stores/auth';

const route = useRoute();
const themeStore = useThemeStore();
const authStore = useAuthStore();

onMounted(() => {
  themeStore.applyTheme(route.name === 'login' ? true : undefined);
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
