<template>
  <router-view />
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import { watch } from 'vue';
import { useThemeStore } from 'src/stores/theme';
const route = useRoute();
const themeStore = useThemeStore();
watch(
  () => route.path,
  (path) => {
    themeStore.setPortalRoute(path.startsWith('/client') && !path.startsWith('/clients'));
  },
  { immediate: true },
);
</script>
