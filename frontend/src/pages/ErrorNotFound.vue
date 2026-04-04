<template>
  <div class="fullscreen text-center q-pa-md flex flex-center">
    <div>
      <div style="font-size: 30vh">404</div>

      <div class="text-h2" style="opacity: 0.4">{{ t('error.notFound.message') }}</div>

      <q-btn
        class="q-mt-xl"
        color="primary"
        unelevated
        to="/"
        :label="t('error.notFound.backHome')"
        no-caps
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
</script>
