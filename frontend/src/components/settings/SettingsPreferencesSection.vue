<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ t('ui.preferences') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ t('settings.preferences.subtitle') }}
      </div>
    </q-card-section>

    <q-card-section class="q-pt-none">
      <div class="form-section-title">{{ t('ui.appearance') }}</div>
      <p class="text-secondary">{{ t('ui.appearanceHint') }}</p>
      <div class="theme-choices" role="group" :aria-label="t('ui.appearance')">
        <button
          v-for="id in themeIds"
          :key="id"
          type="button"
          class="theme-choice"
          :class="{ 'theme-choice--selected': form.themeId === id }"
          :aria-pressed="form.themeId === id"
          @click="form.themeId = id"
          :data-preview-id="id"
        >
          <span class="theme-choice__swatches"><i></i><i></i><i></i></span>
          <span>{{ t('ui.' + id) }}</span
          ><q-icon v-if="form.themeId === id" name="check_circle" />
        </button>
      </div>
      <div class="appearance-sample q-mt-md">
        <div class="row items-center justify-between q-mb-sm">
          <span class="text-caption">{{ t('ui.preview') }}</span
          ><q-badge color="primary">{{ t('tasks.form.status.todo') }}</q-badge>
        </div>
        <strong>{{ t('ui.previewTask') }}</strong>
        <div class="text-caption text-secondary">{{ t('ui.previewMeta') }}</div>
      </div>
      <div class="row q-col-gutter-md q-mt-sm">
        <div class="col-12 col-sm-6">
          <q-select
            v-model="form.density"
            :options="densityOptions"
            :label="t('ui.density')"
            outlined
            dense
            emit-value
            map-options
          />
        </div>
        <div class="col-12 col-sm-6">
          <q-toggle v-model="form.reducedMotion" :label="t('ui.motion')" />
        </div>
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

      <div class="col-12 form-section-title q-mt-md">{{ t('ui.regional') }}</div>
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
        <q-select
          v-model="form.thousandSeparator"
          :label="t('settings.preferences.thousandSeparatorLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="separatorOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <q-select
          v-model="form.decimalSeparator"
          :label="t('settings.preferences.decimalSeparatorLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="separatorOptions"
        />
      </div>

      <div class="col-12 col-md-3">
        <NumericTextInput
          v-model="form.decimalQuantity"
          :label="t('settings.preferences.decimalQuantityLabel')"
          :only-integers="true"
          :decimal-places="0"
        />
        <div class="text-caption q-mt-xs" style="color: var(--text-muted)">
          {{ t('settings.preferences.decimalQuantityHint') }}
        </div>
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
      <span class="text-caption text-secondary q-mr-auto" role="status">{{
        dirty ? t('ui.pending') : ''
      }}</span>
      <q-btn flat :label="t('ui.reset')" :disable="isSaving" @click="resetAppearance" />
      <q-btn
        flat
        :label="t('common.cancel')"
        :disable="isSaving || !dirty"
        @click="cancelChanges"
      />
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
import { computed, onMounted, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { useUnsavedChanges } from 'src/composables/useUnsavedChanges';
import { useI18n } from 'vue-i18n';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import { COMMON_TIMEZONES } from 'src/data/constants';
import { useThemeStore } from 'src/stores/theme';
import { setAppLocale } from 'src/boot/i18n';
import type {
  UserDateFormatPreference,
  UserNumberFormatPreference,
  UpdateUserPreferencesDto,
  UserLanguagePreference,
  UserPreferences,
  UserSettings,
  UserThemePreference,
  ThemeIdentity,
  InterfaceDensity,
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
  'dirty-change': [value: boolean];
}>();

const { t } = useI18n();
const { notify, notifyError } = useUI();
const themeStore = useThemeStore();

const isSaving = ref(false);
const allTimezones = ref<string[]>([]);
const timezoneOptions = ref<SelectOption[]>([]);
const languageOptions = computed(() => [
  { label: t('settings.preferences.language.es'), value: 'es' },
  { label: t('settings.preferences.language.en'), value: 'en' },
  { label: t('settings.preferences.language.pt'), value: 'pt' },
]);
const dateFormatOptions = [
  { label: 'DD/MM/YYYY', value: 'DD/MM/YYYY' },
  { label: 'MM/DD/YYYY', value: 'MM/DD/YYYY' },
  { label: 'YYYY/MM/DD', value: 'YYYY/MM/DD' },
];
const separatorOptions = computed(() => [
  { label: t('settings.preferences.separator.comma'), value: ',' },
  { label: t('settings.preferences.separator.dot'), value: '.' },
]);
const themeOptions = computed(() => [
  { label: t('settings.preferences.theme.system'), value: 'system' },
  { label: t('settings.preferences.theme.dark'), value: 'dark' },
  { label: t('settings.preferences.theme.light'), value: 'light' },
]);

const themeIds: ThemeIdentity[] = ['orbit', 'linen', 'forest'];
const densityOptions = computed(() =>
  ['comfortable', 'compact'].map((value) => ({ label: t('ui.' + value), value })),
);
const savedForm = ref('');
const savedLanguage = ref('es');
const form = reactive({
  themeId: 'orbit' as ThemeIdentity,
  density: 'comfortable' as InterfaceDensity,
  reducedMotion: false,
  theme: 'system' as UserThemePreference,
  timezone: 'UTC',
  language: 'es' as UserLanguagePreference,
  dateFormat: 'DD/MM/YYYY' as UserDateFormatPreference,
  thousandSeparator: ',' as UserNumberFormatPreference['thousand_separator'],
  decimalSeparator: '.' as UserNumberFormatPreference['decimal_separator'],
  decimalQuantity: '2',
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
  const normalizedDecimalQuantity = Number(form.decimalQuantity);

  if (
    !Number.isInteger(normalizedDecimalQuantity) ||
    normalizedDecimalQuantity < 1 ||
    normalizedDecimalQuantity > 4
  ) {
    notify(t('settings.preferences.invalidDecimalQuantity'), 'warning');
    return;
  }

  if (form.thousandSeparator === form.decimalSeparator) {
    notify(t('settings.preferences.invalidNumberSeparators'), 'warning');
    return;
  }

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
      appearance: {
        theme_id: form.themeId,
        density: form.density,
        reduced_motion: form.reducedMotion,
      },
      timezone: form.timezone,
      date_format: form.dateFormat,
      number_format: {
        thousand_separator: form.thousandSeparator,
        decimal_separator: form.decimalSeparator,
        decimal_quantity: normalizedDecimalQuantity,
      },
      language: form.language,
      max_daily_hours: normalizedMaxDailyHours,
    };
    const updated = await UsersService.updatePreferences(preferences);

    themeStore.applyPreferences(updated.preferences);
    savedForm.value = JSON.stringify(form);
    savedLanguage.value = form.language;
    setAppLocale(form.language);

    emit('updated', updated);
    notify(t('settings.preferences.success'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.preferences.error'), error);
  } finally {
    isSaving.value = false;
  }
}

watch(
  () => form.language,
  (newLang) => {
    setAppLocale(newLang);
  },
);

watch(
  () => form.thousandSeparator,
  (newSeparator) => {
    form.decimalSeparator = newSeparator === ',' ? '.' : ',';
  },
);

watch(
  () => form.decimalSeparator,
  (newSeparator) => {
    form.thousandSeparator = newSeparator === ',' ? '.' : ',';
  },
);

watch(
  () => props.preferences,
  (value) => {
    if (!value) return;

    form.themeId = value.appearance?.theme_id ?? 'orbit';
    form.density = value.appearance?.density ?? 'comfortable';
    form.reducedMotion = value.appearance?.reduced_motion ?? false;
    form.theme = value.theme;
    form.language = value.language;
    form.timezone = value.timezone;
    form.dateFormat = value.date_format ?? 'DD/MM/YYYY';
    form.thousandSeparator = value.number_format?.thousand_separator ?? ',';
    form.decimalSeparator = value.number_format?.decimal_separator ?? '.';
    form.decimalQuantity = String(value.number_format?.decimal_quantity ?? 2);
    form.maxDailyHours = String(value.max_daily_hours ?? 8);

    ensureTimezoneOption(value.timezone);
    savedForm.value = JSON.stringify(form);
    savedLanguage.value = value.language;
    themeStore.applyPreferences(value);
  },
  { immediate: true },
);

const dirty = computed(() => savedForm.value !== '' && savedForm.value !== JSON.stringify(form));
watch(dirty, (value) => emit('dirty-change', value), { immediate: true });
watch(
  () => [form.theme, form.themeId, form.density, form.reducedMotion],
  () => {
    themeStore.preview({
      mode: form.theme,
      theme_id: form.themeId,
      density: form.density,
      reduced_motion: form.reducedMotion,
    });
  },
);
function cancelChanges() {
  if (savedForm.value) Object.assign(form, JSON.parse(savedForm.value));
  themeStore.cancelPreview();
  setAppLocale(savedLanguage.value);
}
function resetAppearance() {
  Object.assign(form, {
    theme: 'system',
    themeId: 'orbit',
    density: 'comfortable',
    reducedMotion: false,
  });
}
useUnsavedChanges(() => dirty.value && !isSaving.value);
onBeforeUnmount(() => {
  themeStore.cancelPreview();
  setAppLocale(savedLanguage.value);
});
onMounted(() => {
  loadAvailableTimezones();
});
</script>

<style scoped>
.theme-choices {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}
.theme-choice {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  padding: 16px;
  border-radius: 12px;
  border: 1px solid var(--border-control);
  color: var(--text-primary);
  background: var(--surface-panel);
  cursor: pointer;
  font: inherit;
  text-align: left;
}
.theme-choice--selected {
  border: 2px solid var(--text-accent);
  padding: 15px;
  background: var(--surface-selected);
}
.theme-choice__swatches {
  display: flex;
  width: 100%;
  gap: 4px;
}
.theme-choice__swatches i {
  height: 20px;
  flex: 1;
  border-radius: 4px;
  background: #e6e8f2;
}
.theme-choice__swatches i:nth-child(2) {
  background: #5146c7;
}
.theme-choice__swatches i:nth-child(3) {
  background: #22283b;
}
[data-preview-id='linen'] i:first-child {
  background: #f0eae1;
}
[data-preview-id='linen'] i:nth-child(2) {
  background: #8a4931;
}
[data-preview-id='linen'] i:nth-child(3) {
  background: #362d28;
}
[data-preview-id='forest'] i:first-child {
  background: #e5eee8;
}
[data-preview-id='forest'] i:nth-child(2) {
  background: #24654f;
}
[data-preview-id='forest'] i:nth-child(3) {
  background: #24372d;
}
.appearance-sample {
  padding: 16px;
  background: var(--surface-page);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
}
@media (max-width: 599px) {
  .theme-choices {
    gap: 8px;
  }
  .theme-choice {
    padding: 10px;
  }
  .theme-choice--selected {
    padding: 9px;
  }
}
</style>
