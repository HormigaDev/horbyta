<template>
  <div class="markdown-preview" v-html="html"></div>
</template>

<script setup lang="ts">
interface Props {
  html: string;
}

defineProps<Props>();
</script>
