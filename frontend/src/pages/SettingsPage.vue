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

      <q-card flat bordered>
        <q-card-section class="row items-center q-col-gutter-md">
          <div class="col-12 col-md-8">
            <div class="text-h6">{{ t('settings.storage.title') }}</div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('settings.storage.subtitle') }}
            </div>
          </div>

          <div class="col-12 col-md-4 text-left text-md-right" v-if="storageUsage">
            <div class="text-subtitle2">
              {{
                t('settings.storage.usageLabel', {
                  used: formatStorageBytes(storageUsage.used_bytes),
                  limit: formatStorageBytes(storageUsage.limit_bytes),
                })
              }}
            </div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{
                t('settings.storage.remainingLabel', {
                  remaining: formatStorageBytes(storageUsage.remaining_bytes),
                })
              }}
            </div>
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-linear-progress
            v-if="storageUsage"
            rounded
            size="12px"
            :value="storageUsageRatio"
            :color="storageUsageColor"
            track-color="grey-4"
          />
          <div v-if="storageUsage" class="text-caption q-mt-xs" style="color: var(--text-muted)">
            {{ t('settings.storage.percentLabel', { percent: storageUsagePercent }) }}
          </div>
          <div v-else class="text-caption" style="color: var(--text-muted)">
            {{ t('settings.storage.unavailable') }}
          </div>
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
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import SettingsAppSection from 'src/components/settings/SettingsAppSection.vue';
import SettingsPreferencesSection from 'src/components/settings/SettingsPreferencesSection.vue';
import SettingsUserSection from 'src/components/settings/SettingsUserSection.vue';
import SettingsNotificationsSection from 'src/components/settings/SettingsNotificationsSection.vue';
import { useUI } from 'src/composables/useUI';
import { AttachmentsService } from 'src/services/attachments';
import { UsersService } from 'src/services/users';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import type { AttachmentStorageUsageDto } from 'src/types/attachment';
import type { UserSettings } from 'src/types/user';
import { formatStorageBytes } from 'src/utils/attachments';

const { t } = useI18n();
const { notifyError } = useUI();
const userPreferencesStore = useUserPreferencesStore();

const isBootstrapping = ref(false);
const settings = ref<UserSettings | null>(null);
const storageUsage = ref<AttachmentStorageUsageDto | null>(null);

const storageUsageRatio = computed(() => {
  if (!storageUsage.value) return 0;

  const limit = storageUsage.value.limit_bytes;
  if (limit <= 0) return 0;

  const ratio = storageUsage.value.used_bytes / limit;
  return Math.min(1, Math.max(0, ratio));
});

const storageUsagePercent = computed(() => Math.round(storageUsageRatio.value * 100));

const storageUsageColor = computed(() => {
  if (storageUsageRatio.value >= 0.9) return 'negative';
  if (storageUsageRatio.value >= 0.7) return 'warning';
  return 'positive';
});

async function loadSettings() {
  isBootstrapping.value = true;

  try {
    const loadedSettings = await UsersService.getMe();
    settings.value = loadedSettings;
    userPreferencesStore.applySettings(loadedSettings);

    try {
      storageUsage.value = await AttachmentsService.getStorageUsage();
    } catch (error: unknown) {
      storageUsage.value = null;
      notifyError(t('settings.storage.loadError'), error);
    }
  } catch (error: unknown) {
    notifyError(t('settings.page.loadError'), error);
    settings.value = null;
    storageUsage.value = null;
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
