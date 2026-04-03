<template>
  <q-page class="q-pa-md">
    <div class="settings-container q-gutter-y-md">
      <q-card flat bordered>
        <q-card-section class="row items-center justify-between q-col-gutter-md">
          <div>
            <div class="text-h5">Configuraciones</div>
            <div class="text-caption" style="color: var(--text-muted)">
              Define tus preferencias, datos de usuario y notificaciones.
            </div>
          </div>
          <q-btn
            flat
            icon="refresh"
            label="Recargar"
            color="primary"
            :loading="isBootstrapping"
            @click="loadSettings"
          />
        </q-card-section>
      </q-card>

      <template v-if="settings">
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
          <div class="text-subtitle1">No se pudo cargar la configuración</div>
          <div class="text-caption" style="color: var(--text-muted)">
            Intenta recargar para volver a sincronizar la información.
          </div>
        </q-card-section>
      </q-card>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import SettingsPreferencesSection from 'src/components/settings/SettingsPreferencesSection.vue';
import SettingsUserSection from 'src/components/settings/SettingsUserSection.vue';
import SettingsNotificationsSection from 'src/components/settings/SettingsNotificationsSection.vue';
import { useUI } from 'src/composables/useUI';
import { UsersService } from 'src/services/users';
import type { UserSettings } from 'src/types/user';

const { notifyError } = useUI();

const isBootstrapping = ref(false);
const settings = ref<UserSettings | null>(null);

async function loadSettings() {
  isBootstrapping.value = true;

  try {
    settings.value = await UsersService.getMe();
  } catch (error: unknown) {
    notifyError('No se pudo cargar la configuración de usuario', error);
    settings.value = null;
  } finally {
    isBootstrapping.value = false;
  }
}

function handleSettingsUpdated(updated: UserSettings) {
  settings.value = updated;
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
