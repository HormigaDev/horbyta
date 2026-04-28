<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ t('settings.preferences.title') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ t('settings.preferences.subtitle') }}
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row q-col-gutter-md">
      <div class="col-12 col-md-3">
        <q-select
          v-model="form.theme"
          :label="t('settings.preferences.themeLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="themeOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <q-select
          v-model="form.language"
          :label="t('settings.preferences.languageLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="languageOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <q-select
          v-model="form.dateFormat"
          :label="t('settings.preferences.dateFormatLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="dateFormatOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <q-select
          v-model="form.timezone"
          :label="t('settings.preferences.timezoneLabel')"
          outlined
          dense
          emit-value
          map-options
          use-input
          input-debounce="0"
          :options="timezoneOptions"
          @filter="filterTimezoneOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <NumericTextInput
          v-model="form.maxDailyHours"
          :label="t('settings.preferences.maxDailyHoursLabel')"
          :only-integers="true"
          :decimal-places="0"
        />
        <div class="text-caption q-mt-xs" style="color: var(--text-muted)">
          {{ t('settings.preferences.maxDailyHoursHint') }}
        </div>
      </div>
    </q-card-section>

    <q-card-actions align="right">
      <q-btn
        color="primary"
        unelevated
        :label="t('settings.preferences.saveButton')"
        :loading="isSaving"
        @click="savePreferences"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import { COMMON_TIMEZONES } from 'src/data/constants';
import { useThemeStore } from 'src/stores/theme';
import type {
  UserDateFormatPreference,
  UpdateUserPreferencesDto,
  UserLanguagePreference,
  UserPreferences,
  UserSettings,
  UserThemePreference,
} from 'src/types/user';

interface SelectOption {
  label: string;
  value: string;
}

type QuasarFilterUpdateFn = (callbackFn: () => void) => void;

const props = defineProps<{
  preferences: UserPreferences | null;
}>();

const emit = defineEmits<{
  updated: [value: UserSettings];
}>();

const { t } = useI18n();
const { notify, notifyError } = useUI();
const themeStore = useThemeStore();

const isSaving = ref(false);
const allTimezones = ref<string[]>([]);
const timezoneOptions = ref<SelectOption[]>([]);
const languageOptions = [
  { label: t('settings.preferences.language.es'), value: 'es' },
  { label: t('settings.preferences.language.en'), value: 'en' },
  { label: t('settings.preferences.language.pt'), value: 'pt' },
];
const dateFormatOptions = [
  { label: 'DD/MM/YYYY', value: 'DD/MM/YYYY' },
  { label: 'MM/DD/YYYY', value: 'MM/DD/YYYY' },
  { label: 'YYYY/MM/DD', value: 'YYYY/MM/DD' },
];
const themeOptions = [
  { label: t('settings.preferences.theme.system'), value: 'system' },
  { label: t('settings.preferences.theme.dark'), value: 'dark' },
  { label: t('settings.preferences.theme.light'), value: 'light' },
];

const form = reactive({
  theme: 'system' as UserThemePreference,
  timezone: 'UTC',
  language: 'es' as UserLanguagePreference,
  dateFormat: 'DD/MM/YYYY' as UserDateFormatPreference,
  maxDailyHours: '8',
});

function toTimezoneOptions(values: string[]): SelectOption[] {
  return values.map((timezone) => ({ label: timezone, value: timezone }));
}

function loadAvailableTimezones() {
  const maybeIntl = Intl as unknown as { supportedValuesOf?: (key: 'timeZone') => string[] };

  if (typeof maybeIntl.supportedValuesOf === 'function') {
    const values = maybeIntl.supportedValuesOf('timeZone');
    allTimezones.value = [...values].sort((a, b) => a.localeCompare(b));
  } else {
    allTimezones.value = [...COMMON_TIMEZONES].sort((a, b) => a.localeCompare(b));
  }

  timezoneOptions.value = toTimezoneOptions(allTimezones.value);
}

function ensureTimezoneOption(value: string) {
  if (!allTimezones.value.includes(value)) {
    allTimezones.value = [...allTimezones.value, value].sort((a, b) => a.localeCompare(b));
    timezoneOptions.value = toTimezoneOptions(allTimezones.value);
  }
}

function filterTimezoneOptions(value: string, update: QuasarFilterUpdateFn) {
  update(() => {
    const needle = value.trim().toLowerCase();

    if (!needle) {
      timezoneOptions.value = toTimezoneOptions(allTimezones.value);
      return;
    }

    const filtered = allTimezones.value.filter((timezone) =>
      timezone.toLowerCase().includes(needle),
    );
    timezoneOptions.value = toTimezoneOptions(filtered);
  });
}

async function savePreferences() {
  if (!form.timezone.trim()) {
    notify(t('settings.preferences.invalidTimezone'), 'warning');
    return;
  }

  const normalizedMaxDailyHours = Number(form.maxDailyHours);
  if (
    !Number.isInteger(normalizedMaxDailyHours) ||
    normalizedMaxDailyHours < 1 ||
    normalizedMaxDailyHours > 14
  ) {
    notify(t('settings.preferences.invalidMaxDailyHours'), 'warning');
    return;
  }

  isSaving.value = true;

  try {
    const preferences: UpdateUserPreferencesDto = {
      theme: form.theme,
      timezone: form.timezone,
      date_format: form.dateFormat,
      language: form.language,
      max_daily_hours: normalizedMaxDailyHours,
    };
    const updated = await UsersService.updatePreferences(preferences);

    let isDarkMode = updated.preferences.theme === 'dark';
    if (updated.preferences.theme === 'system') {
      isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    themeStore.setThemePreference(isDarkMode);

    emit('updated', updated);
    notify(t('settings.preferences.success'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.preferences.error'), error);
  } finally {
    isSaving.value = false;
  }
}

watch(
  () => props.preferences,
  (value) => {
    if (!value) return;

    form.theme = value.theme;
    form.language = value.language;
    form.timezone = value.timezone;
    form.dateFormat = value.date_format ?? 'DD/MM/YYYY';
    form.maxDailyHours = String(value.max_daily_hours ?? 8);

    ensureTimezoneOption(value.timezone);
  },
  { immediate: true },
);

onMounted(() => {
  loadAvailableTimezones();
});
</script>
