<template>
  <q-layout view="lHh Lpr lFf"
    ><q-page-container
      ><q-page class="auth-page">
        <div class="auth-tools">
          <q-select
            :model-value="locale"
            :options="languages"
            emit-value
            map-options
            outlined
            dense
            :label="t('ui.language')"
            @update:model-value="setAppLocale"
          />
          <AppearanceMenu v-if="portal || !auth.isAuthenticated" />
        </div>
        <main class="auth-shell">
          <aside class="auth-intro">
            <div class="auth-brand">
              <img src="/logo.png" width="48" height="48" alt="" />Horbyta
            </div>
            <div class="page-header__eyebrow">
              {{ t(portal ? 'ui.clientPortal' : 'ui.workspace') }}
            </div>
            <h1>{{ t(portal ? 'ui.portalWelcome' : 'ui.authWelcome') }}</h1>
            <p>{{ t(portal ? 'ui.portalIntro' : 'ui.authIntro') }}</p>
            <div class="auth-feature">
              <q-icon :name="portal ? 'fact_check' : 'checklist'" />{{
                t(portal ? 'ui.portalFeature' : 'ui.authFeature')
              }}
            </div>
          </aside>
          <q-card flat bordered class="auth-card">
            <q-card-section
              ><h2 class="auth-title">{{ title }}</h2>
              <p class="text-secondary q-mb-none">{{ subtitle }}</p></q-card-section
            >
            <slot />
          </q-card>
        </main> </q-page></q-page-container
  ></q-layout>
</template>
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import AppearanceMenu from './AppearanceMenu.vue';
import { setAppLocale } from 'src/boot/i18n';
import { useAuthStore } from 'src/stores/auth';
defineProps<{ title: string; subtitle: string; portal?: boolean }>();
const { t, locale } = useI18n();
const auth = useAuthStore();
const languages = [
  { value: 'es', label: 'Español' },
  { value: 'en', label: 'English' },
  { value: 'pt', label: 'Português' },
];
</script>
