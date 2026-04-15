<template>
  <q-page class="q-pa-md">
    <div class="settings-container q-gutter-y-md">
      <q-card flat bordered>
        <q-card-section class="row items-center justify-between q-col-gutter-md">
          <div>
            <div class="text-h5">{{ t('settings.page.title') }}</div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('settings.page.subtitle') }}
            </div>
          </div>
          <q-btn
            flat
            icon="refresh"
            :label="t('settings.page.reload')"
            color="primary"
            :loading="isBootstrapping"
            @click="loadSettings"
          />
        </q-card-section>
      </q-card>

      <template v-if="settings">
        <SettingsAppSection />

        <SettingsPreferencesSection
          :preferences="settings.preferences"
          @updated="handleSettingsUpdated"
        />

        <SettingsUserSection :user="settings" @updated="handleSettingsUpdated" />

        <SettingsNotificationsSection
          :notifications="settings.preferences.notifications"
          @updated="handleSettingsUpdated"
        />
      </template>

      <q-card v-else-if="!isBootstrapping" flat bordered>
        <q-card-section>
          <div class="text-subtitle1">{{ t('settings.page.emptyTitle') }}</div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('settings.page.emptySubtitle') }}
          </div>
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import SettingsAppSection from 'src/components/settings/SettingsAppSection.vue';
import SettingsPreferencesSection from 'src/components/settings/SettingsPreferencesSection.vue';
import SettingsUserSection from 'src/components/settings/SettingsUserSection.vue';
import SettingsNotificationsSection from 'src/components/settings/SettingsNotificationsSection.vue';
import { useUI } from 'src/composables/useUI';
import { UsersService } from 'src/services/users';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { UserSettings } from 'src/types/user';

const { t } = useI18n();
const { notifyError } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const isBootstrapping = ref(false);
const settings = ref<UserSettings | null>(null);

async function loadSettings() {
  isBootstrapping.value = true;

  try {
    const loadedSettings = await UsersService.getMe();
    settings.value = loadedSettings;
    userPreferencesStore.applySettings(loadedSettings);
  } catch (error: unknown) {
    notifyError(t('settings.page.loadError'), error);
    settings.value = null;
  } finally {
    isBootstrapping.value = false;
  }
}

function handleSettingsUpdated(updated: UserSettings) {
  settings.value = updated;
  userPreferencesStore.applySettings(updated);
}

onMounted(() => {
  void loadSettings();
});
</script>

<style scoped>
.settings-container {
  max-width: 1200px;
  margin: 0 auto;
}
</style>
