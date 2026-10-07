<template>
  <main class="not-found-page">
    <div class="not-found-card">
      <q-icon name="explore_off" size="56px" color="primary" />
      <div class="page-header__eyebrow q-mt-md">404</div>
      <h1>{{ t('error.notFound.message') }}</h1>
      <p class="text-secondary">{{ t('ui.notFoundHint') }}</p>
      <q-btn
        color="primary"
        unelevated
        :to="portal ? '/client' : '/'"
        :label="t('error.notFound.backHome')"
        icon-right="arrow_forward"
      />
    </div>
  </main>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
const route = useRoute();
const portal = computed(() => route.path === '/client' || route.path.startsWith('/client/'));
</script>
