<template>
  <header class="page-header">
    <div>
      <div v-if="eyebrow" class="page-header__eyebrow">{{ eyebrow }}</div>
      <h1 class="page-header__title">{{ title }}</h1>
      <div v-if="subtitle" class="page-header__subtitle">{{ subtitle }}</div>
    </div>
    <div class="row items-center q-gutter-sm">
      <slot name="actions"><slot /></slot>
    </div>
  </header>
</template>
<script setup lang="ts">
defineProps<{ title: string; subtitle?: string; eyebrow?: string }>();
</script>
