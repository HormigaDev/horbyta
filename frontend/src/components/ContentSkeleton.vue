<template>
  <div
    class="content-skeleton"
    :class="`content-skeleton--${variant}`"
    aria-busy="true"
    role="status"
  >
    <span class="sr-only">{{ t('common.loading') }}</span>
    <div aria-hidden="true">
      <template v-if="variant === 'dashboard' || variant === 'board'">
        <div class="row q-col-gutter-md">
          <div
            v-for="index in variant === 'board' ? 4 : 3"
            :key="index"
            class="col-12 col-sm-6"
            :class="variant === 'board' ? 'col-lg-3' : 'col-md-4'"
          >
            <div class="skeleton-panel">
              <q-skeleton type="text" width="55%" height="32px" />
              <q-skeleton v-if="variant === 'dashboard'" type="text" width="38%" height="56px" />
              <q-skeleton
                v-for="row in variant === 'board' ? 3 : 1"
                :key="row"
                type="rect"
                :height="variant === 'board' ? '112px' : '24px'"
                class="q-mt-md"
              />
            </div>
          </div>
        </div>
        <div v-if="variant === 'dashboard'" class="skeleton-panel q-mt-md">
          <q-skeleton type="rect" height="260px" />
        </div>
      </template>
      <div v-else-if="variant === 'chart'" class="skeleton-chart">
        <q-skeleton type="rect" height="240px" />
        <q-skeleton type="text" width="60%" class="q-mt-md" />
      </div>
      <div v-else-if="variant === 'list' || variant === 'table'" class="skeleton-rows">
        <div v-for="index in rows" :key="index" class="skeleton-row">
          <q-skeleton v-if="variant === 'list'" type="QAvatar" size="36px" />
          <div class="col">
            <q-skeleton type="text" width="65%" /><q-skeleton type="text" width="40%" />
          </div>
          <q-skeleton type="rect" width="60px" height="24px" />
        </div>
      </div>
      <template v-else>
        <div
          v-for="section in variant === 'form' ? 2 : 3"
          :key="section"
          class="skeleton-panel q-mb-md"
        >
          <q-skeleton type="text" width="45%" height="36px" class="q-mb-md" />
          <div v-if="variant === 'form'" class="row q-col-gutter-md">
            <div v-for="field in 4" :key="field" class="col-12 col-sm-6">
              <q-skeleton type="QInput" class="q-mb-md" />
            </div>
          </div>
          <template v-else
            ><q-skeleton
              v-for="line in 3"
              :key="line"
              type="text"
              :width="line === 3 ? '65%' : '100%'"
              height="24px"
          /></template>
        </div>
      </template>
    </div>
  </div>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
withDefaults(
  defineProps<{
    variant?: 'detail' | 'form' | 'list' | 'table' | 'dashboard' | 'board' | 'chart';
    rows?: number;
  }>(),
  { variant: 'detail', rows: 5 },
);
</script>
<style scoped>
.content-skeleton {
  width: 100%;
  min-width: 0;
}
.skeleton-panel {
  padding: var(--panel-padding);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-panel);
  background: var(--surface-panel);
}
.skeleton-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 8px;
  border-bottom: 1px solid var(--border-subtle);
  min-height: 64px;
}
.skeleton-chart {
  padding: 8px;
}
@media (prefers-reduced-motion: reduce) {
  :deep(.q-skeleton) {
    animation: none !important;
  }
  :deep(.q-skeleton::after) {
    animation: none !important;
  }
}
</style>
