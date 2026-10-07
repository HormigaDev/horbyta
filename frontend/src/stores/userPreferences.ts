import { defineStore } from 'pinia';
import { ref } from 'vue';
import { UsersService } from 'src/services/users';
import { useUiStateStore } from 'src/stores/uiState';
import { useThemeStore } from 'src/stores/theme';
import { setAppLocale } from 'src/boot/i18n';
import type { PinnedTask, UserPreferences, UserSettings } from 'src/types/user';
import {
  DEFAULT_DATE_FORMAT,
  DEFAULT_TIMEZONE,
  normalizeDateFormat,
  normalizeTimezone,
} from 'src/utils/dateFormatting';
import {
  DEFAULT_DECIMAL_QUANTITY,
  DEFAULT_DECIMAL_SEPARATOR,
  DEFAULT_THOUSAND_SEPARATOR,
  normalizeNumberFormat,
} from 'src/utils/numberFormatting';
import type { UserNumberFormatPreference } from 'src/types/user';

const PINNED_SCOPE = 'pinned' as const;
const MAX_PINNED_TASKS = 10;

export const useUserPreferencesStore = defineStore('userPreferences', () => {
  const timezone = ref(DEFAULT_TIMEZONE);
  const dateFormat = ref(DEFAULT_DATE_FORMAT);
  const thousandSeparator = ref<UserNumberFormatPreference['thousand_separator']>(
    DEFAULT_THOUSAND_SEPARATOR,
  );
  const decimalSeparator =
    ref<UserNumberFormatPreference['decimal_separator']>(DEFAULT_DECIMAL_SEPARATOR);
  const decimalQuantity = ref(DEFAULT_DECIMAL_QUANTITY);
  const groupNotifications = ref(false);
  const pinnedTasks = ref<PinnedTask[]>([]);
  const isLoaded = ref(false);
  const isLoading = ref(false);

  const uiState = useUiStateStore();
  const themeStore = useThemeStore();

  function applyPreferences(preferences: UserPreferences | null | undefined) {
    timezone.value = normalizeTimezone(preferences?.timezone);
    dateFormat.value = normalizeDateFormat(preferences?.date_format);
    const numberFormat = normalizeNumberFormat(preferences?.number_format);
    thousandSeparator.value = numberFormat.thousand_separator;
    decimalSeparator.value = numberFormat.decimal_separator;
    decimalQuantity.value = numberFormat.decimal_quantity;
    groupNotifications.value = preferences?.notifications?.group_notifications ?? false;
    themeStore.applyPreferences(preferences);
    if (preferences?.language) {
      setAppLocale(preferences.language);
    }
    isLoaded.value = true;
  }

  function applySettings(settings: UserSettings | null | undefined) {
    applyPreferences(settings?.preferences);
  }

  function refreshPinnedFromStore() {
    const entries = uiState.listScope<PinnedTask>(PINNED_SCOPE);
    pinnedTasks.value = [...entries]
      .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
      .slice(0, MAX_PINNED_TASKS)
      .map((entry) => entry.value);
  }

  async function loadPinnedTasks() {
    await uiState.ensureScopeLoaded(PINNED_SCOPE);
    refreshPinnedFromStore();
  }

  function resetToDefaults() {
    timezone.value = DEFAULT_TIMEZONE;
    dateFormat.value = DEFAULT_DATE_FORMAT;
    thousandSeparator.value = DEFAULT_THOUSAND_SEPARATOR;
    decimalSeparator.value = DEFAULT_DECIMAL_SEPARATOR;
    decimalQuantity.value = DEFAULT_DECIMAL_QUANTITY;
    groupNotifications.value = false;
    pinnedTasks.value = [];
    isLoaded.value = false;
  }

  function isPinned(taskId: string): boolean {
    return pinnedTasks.value.some((t) => t.id === taskId);
  }

  async function pinTask(task: PinnedTask): Promise<void> {
    await uiState.setValue<PinnedTask>(PINNED_SCOPE, task.id, {
      id: task.id,
      record_number: task.record_number,
      title: task.title,
      project_name: task.project_name,
    });
    refreshPinnedFromStore();
  }

  async function unpinTask(taskId: string): Promise<void> {
    await uiState.removeValue(PINNED_SCOPE, taskId);
    refreshPinnedFromStore();
  }

  async function ensureLoaded() {
    if (isLoaded.value || isLoading.value) {
      return;
    }

    isLoading.value = true;

    try {
      const settings = await UsersService.getMe();
      applyPreferences(settings.preferences);
      try {
        await loadPinnedTasks();
      } catch (error) {
        console.error('Error loading pinned tasks:', error);
        pinnedTasks.value = [];
      }
    } catch {
      timezone.value = DEFAULT_TIMEZONE;
      dateFormat.value = DEFAULT_DATE_FORMAT;
      thousandSeparator.value = DEFAULT_THOUSAND_SEPARATOR;
      decimalSeparator.value = DEFAULT_DECIMAL_SEPARATOR;
      decimalQuantity.value = DEFAULT_DECIMAL_QUANTITY;
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
    thousandSeparator,
    decimalSeparator,
    decimalQuantity,
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
