import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UsersService } from 'src/services/users';
import type { PinnedTask, UserPreferences, UserSettings } from 'src/types/user';
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
  const pinnedTasks = ref<PinnedTask[]>([]);
  const isLoaded = ref(false);
  const isLoading = ref(false);

  function applyPreferences(preferences: UserPreferences | null | undefined) {
    timezone.value = normalizeTimezone(preferences?.timezone);
    dateFormat.value = normalizeDateFormat(preferences?.date_format);
    groupNotifications.value = preferences?.notifications?.group_notifications ?? false;
    pinnedTasks.value = preferences?.pinned_tasks ?? [];
    isLoaded.value = true;
  }

  function applySettings(settings: UserSettings | null | undefined) {
    applyPreferences(settings?.preferences);
  }

  function resetToDefaults() {
    timezone.value = DEFAULT_TIMEZONE;
    dateFormat.value = DEFAULT_DATE_FORMAT;
    groupNotifications.value = false;
    pinnedTasks.value = [];
    isLoaded.value = false;
  }

  function isPinned(taskId: string): boolean {
    return pinnedTasks.value.some((t) => t.id === taskId);
  }

  async function pinTask(task: PinnedTask): Promise<void> {
    const updated = await UsersService.pinTask({
      id: task.id,
      record_number: task.record_number,
      title: task.title,
      project_name: task.project_name,
    });
    applyPreferences(updated.preferences);
  }

  async function unpinTask(taskId: string): Promise<void> {
    const updated = await UsersService.unpinTask(taskId);
    applyPreferences(updated.preferences);
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
      pinnedTasks.value = [];
      isLoaded.value = true;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    timezone,
    dateFormat,
    groupNotifications,
    pinnedTasks,
    isLoaded,
    isLoading,
    applyPreferences,
    applySettings,
    resetToDefaults,
    isPinned,
    pinTask,
    unpinTask,
    ensureLoaded,
  };
});
