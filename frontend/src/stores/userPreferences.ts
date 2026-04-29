import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UsersService } from 'src/services/users';
import type { UserPreferences, UserSettings } from 'src/types/user';
import {
  DEFAULT_DATE_FORMAT,
  DEFAULT_TIMEZONE,
  normalizeDateFormat,
  normalizeTimezone,
} from 'src/utils/dateFormatting';

export const useUserPreferencesStore = defineStore('userPreferences', () => {
  const timezone = ref(DEFAULT_TIMEZONE);
  const dateFormat = ref(DEFAULT_DATE_FORMAT);
  const groupNotifications = ref(false);
  const isLoaded = ref(false);
  const isLoading = ref(false);

  function applyPreferences(preferences: UserPreferences | null | undefined) {
    timezone.value = normalizeTimezone(preferences?.timezone);
    dateFormat.value = normalizeDateFormat(preferences?.date_format);
    groupNotifications.value = preferences?.notifications?.group_notifications ?? false;
    isLoaded.value = true;
  }

  function applySettings(settings: UserSettings | null | undefined) {
    applyPreferences(settings?.preferences);
  }

  function resetToDefaults() {
    timezone.value = DEFAULT_TIMEZONE;
    dateFormat.value = DEFAULT_DATE_FORMAT;
    groupNotifications.value = false;
    isLoaded.value = false;
  }

  async function ensureLoaded() {
    if (isLoaded.value || isLoading.value) {
      return;
    }

    isLoading.value = true;

    try {
      const settings = await UsersService.getMe();
      applyPreferences(settings.preferences);
    } catch {
      timezone.value = DEFAULT_TIMEZONE;
      dateFormat.value = DEFAULT_DATE_FORMAT;
      groupNotifications.value = false;
      isLoaded.value = true;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    timezone,
    dateFormat,
    groupNotifications,
    isLoaded,
    isLoading,
    applyPreferences,
    applySettings,
    resetToDefaults,
    ensureLoaded,
  };
});
