<template>
  <q-btn flat round icon="palette" :aria-label="t('ui.appearance')">
    <q-tooltip>{{ t('ui.appearance') }}</q-tooltip>
    <q-menu
      ><div class="appearance-menu q-pa-md">
        <div class="text-subtitle1 q-mb-sm">{{ t('ui.appearance') }}</div>
        <div class="theme-menu-options">
          <q-btn
            v-for="id in identities"
            :key="id"
            outline
            :color="theme.appearance.theme_id === id ? 'primary' : undefined"
            :aria-pressed="theme.appearance.theme_id === id"
            :label="t('ui.' + id)"
            @click="theme.commit({ ...theme.appearance, theme_id: id })"
          />
        </div>
        <q-select
          class="q-mt-md"
          outlined
          :label="t('ui.colorMode')"
          :model-value="theme.appearance.mode"
          :options="modes"
          emit-value
          map-options
          @update:model-value="(value) => theme.commit({ ...theme.appearance, mode: value })"
        />
        <q-select
          class="q-mt-md"
          outlined
          :label="t('ui.density')"
          :model-value="theme.appearance.density"
          :options="densities"
          emit-value
          map-options
          @update:model-value="(value) => theme.commit({ ...theme.appearance, density: value })"
        />
        <q-toggle
          class="q-mt-sm"
          :label="t('ui.motion')"
          :model-value="theme.appearance.reduced_motion"
          @update:model-value="
            (value) => theme.commit({ ...theme.appearance, reduced_motion: value })
          "
        /></div
    ></q-menu>
  </q-btn>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useThemeStore } from 'src/stores/theme';
const { t } = useI18n();
const theme = useThemeStore();
const identities = ['orbit', 'linen', 'forest'] as const;
const modes = computed(() =>
  ['system', 'light', 'dark'].map((value) => ({
    value,
    label: t('settings.preferences.theme.' + value),
  })),
);
const densities = computed(() =>
  ['comfortable', 'compact'].map((value) => ({ value, label: t('ui.' + value) })),
);
</script>
