<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">Preferencias</div>
      <div class="text-caption" style="color: var(--text-muted)">
        Estas opciones se guardan en tu perfil y se podrán usar en futuras features.
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row q-col-gutter-md">
      <div class="col-12 col-md-4">
        <q-select
          v-model="form.theme"
          label="Tema"
          outlined
          dense
          emit-value
          map-options
          :options="THEME_OPTIONS"
        />
      </div>

      <div class="col-12 col-md-4">
        <q-select
          v-model="form.language"
          label="Idioma"
          outlined
          dense
          emit-value
          map-options
          :options="LANGUAGE_OPTIONS"
        />
      </div>

      <div class="col-12 col-md-4">
        <q-select
          v-model="form.timezone"
          label="Timezone"
          outlined
          dense
          emit-value
          map-options
          use-input
          fill-input
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
        label="Guardar preferencias"
        :loading="isSaving"
        @click="savePreferences"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import { COMMON_TIMEZONES, THEME_OPTIONS, LANGUAGE_OPTIONS } from 'src/data/constants';
import { useThemeStore } from 'src/stores/theme';
import type {
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

const { notify, notifyError } = useUI();
const themeStore = useThemeStore();

const isSaving = ref(false);
const allTimezones = ref<string[]>([]);
const timezoneOptions = ref<SelectOption[]>([]);

const form = reactive({
  theme: 'system' as UserThemePreference,
  timezone: 'UTC',
  language: 'es' as UserLanguagePreference,
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
    notify('Debes seleccionar una timezone válida', 'warning');
    return;
  }

  isSaving.value = true;

  try {
    const updated = await UsersService.updatePreferences({
      theme: form.theme,
      timezone: form.timezone,
      language: form.language,
    });

    let isDarkMode = updated.preferences.theme === 'dark';
    if (updated.preferences.theme === 'system') {
      isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    themeStore.applyTheme(isDarkMode);

    emit('updated', updated);
    notify('Preferencias actualizadas', 'positive');
  } catch (error: unknown) {
    notifyError('No se pudieron guardar las preferencias', error);
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

    ensureTimezoneOption(value.timezone);
  },
  { immediate: true },
);

onMounted(() => {
  loadAvailableTimezones();
});
</script>
