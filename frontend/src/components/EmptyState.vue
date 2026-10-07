<template>
  <div class="empty-state" :role="error ? 'alert' : 'status'">
    <q-icon :name="icon || (error ? 'cloud_off' : 'inbox')" size="30px" class="empty-state__icon" />
    <h2>{{ title }}</h2>
    <div v-if="hint">{{ hint }}</div>
    <slot />
  </div>
</template>
<script setup lang="ts">
defineProps<{ title: string; hint?: string | undefined; icon?: string; error?: boolean }>();
</script>
