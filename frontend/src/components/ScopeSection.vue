<template>
  <section class="scope-section">
    <header class="scope-section__header">
      <q-icon :name="icon" size="20px" aria-hidden="true" />
      <h3>{{ title }}</h3>
    </header>
    <div class="scope-section__body"><slot /></div>
  </section>
</template>
<script setup lang="ts">
withDefaults(defineProps<{ title: string; icon?: string }>(), { icon: 'subject' });
</script>
<style scoped>
.scope-section {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-panel);
  overflow: hidden;
  margin-top: 20px;
  background: var(--surface-panel);
}
.scope-section__header {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--surface-nav);
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-subtle);
  color: var(--text-accent);
}
.scope-section__header h3 {
  margin: 0;
  font-size: 15px;
  line-height: 1.5;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.scope-section__body {
  padding: 20px;
}
.scope-section__body :deep(.markdown-preview) {
  padding: 0;
  min-height: 0;
}
.scope-section__body :deep(.markdown-container) {
  min-height: 0;
}
.scope-section__body :deep(.markdown-preview > :first-child) {
  margin-top: 0;
}
.scope-section__body :deep(.markdown-preview h1) {
  font-size: 1.75rem;
  line-height: 1.35;
}
.scope-section__body :deep(.markdown-preview h2) {
  font-size: 1.4rem;
  line-height: 1.4;
}
.scope-section__body :deep(.markdown-preview h3) {
  font-size: 1.15rem;
  line-height: 1.5;
}
@media (max-width: 599px) {
  .scope-section__header,
  .scope-section__body {
    padding: 14px;
  }
}
</style>
