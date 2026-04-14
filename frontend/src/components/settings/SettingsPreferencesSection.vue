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
      <div class="col-12 col-md-6">
        <q-select
          v-model="form.aiModel"
          :label="t('settings.preferences.aiModelLabel')"
          outlined
          dense
          emit-value
          map-options
          :options="aiModelOptions"
        />
        <div class="text-caption q-mt-xs" style="color: var(--q-warning)">
          {{ $t('settings.preferences.aiModelInfo') }}
        </div>
      </div>

      <div class="col-12 col-md-6">
        <q-input
          v-model="form.aiApiKey"
          :label="t('settings.preferences.aiApiKeyLabel')"
          outlined
          dense
          :type="showApiKey ? 'text' : 'password'"
          autocomplete="off"
        >
          <template #append>
            <q-btn
              flat
              round
              dense
              :icon="showApiKey ? 'visibility_off' : 'visibility'"
              @click="showApiKey = !showApiKey"
            />
          </template>
        </q-input>
        <div class="text-caption q-mt-xs" style="color: var(--text-muted)">
          {{
            props.preferences?.ai.api_key_configured
              ? t('settings.preferences.aiApiKeyConfigured')
              : t('settings.preferences.aiApiKeyMissing')
          }}
        </div>
      </div>

      <div class="col-12 col-md-4">
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

      <div class="col-12 col-md-4">
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

      <div class="col-12 col-md-4">
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
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import { COMMON_TIMEZONES } from 'src/data/constants';
import { useThemeStore } from 'src/stores/theme';
import type {
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
const showApiKey = ref(false);
const allTimezones = ref<string[]>([]);
const timezoneOptions = ref<SelectOption[]>([]);
const aiModelOptions = [
  { label: 'Gemini 2.5 Flash', value: 'google/gemini-2.5-flash' },
  { label: 'Gemini 2.5 Pro', value: 'google/gemini-2.5-pro' },
  { label: 'Gemini 3 Flash (preview)', value: 'google/gemini-3-flash-preview' },
  { label: 'Gemini 3.1 Pro (preview)', value: 'google/gemini-3.1-pro-preview' },
  { label: 'Claude 3 Haiku', value: 'anthropic/claude-3-haiku' },
  { label: 'Claude Haiku 4.5', value: 'anthropic/claude-haiku-4.5' },
  { label: 'Claude Sonnet 4.5', value: 'anthropic/claude-sonnet-4.5' },
  { label: 'Claude Sonnet 4.6', value: 'anthropic/claude-sonnet-4.6' },
  { label: 'Claude Opus 4.5', value: 'anthropic/claude-opus-4.5' },
  { label: 'Claude Opus 4.6', value: 'anthropic/claude-opus-4.6' },
  { label: 'ChatGPT Codex 5.x', value: 'openai/gpt-5-codex' },
];
const languageOptions = [
  { label: t('settings.preferences.language.es'), value: 'es' },
  { label: t('settings.preferences.language.en'), value: 'en' },
  { label: t('settings.preferences.language.pt'), value: 'pt' },
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
  aiModel: 'google/gemini-2.5-flash',
  aiApiKey: '',
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

  isSaving.value = true;

  try {
    const preferences: UpdateUserPreferencesDto = {
      theme: form.theme,
      timezone: form.timezone,
      language: form.language,
      ai_model: form.aiModel,
    };
    const apiKey = form.aiApiKey.trim();
    apiKey && (preferences.ai_api_key = apiKey);
    const updated = await UsersService.updatePreferences(preferences);

    let isDarkMode = updated.preferences.theme === 'dark';
    if (updated.preferences.theme === 'system') {
      isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    themeStore.setThemePreference(isDarkMode);

    emit('updated', updated);
    form.aiApiKey = '';
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
    form.aiModel = value.ai.model;
    form.aiApiKey = '';

    ensureTimezoneOption(value.timezone);
  },
  { immediate: true },
);

onMounted(() => {
  loadAvailableTimezones();
});
</script>
