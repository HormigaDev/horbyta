import { ref, computed, onMounted } from 'vue';
import { UiStateService } from 'src/services/uiState';
import { useI18n } from 'vue-i18n';
import { isApiHttpError } from 'src/boot/fetch';
import { useUI } from 'src/composables/useUI';

export type FormPrefKey = 'task' | 'project' | 'milestone' | 'client';

const SCOPE = 'form_pref' as const;
const MAX_PREFS = 10;

export interface FormPreference {
  id: string;
  name: string;
  fields: Record<string, unknown>;
}

interface FormPreferenceState {
  items: FormPreference[];
  default_id: string | null;
}

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function useFormPreferences(formKey: FormPrefKey) {
  const { notifyError } = useUI();
  const { t } = useI18n();

  const isLoading = ref(false);
  const isSaving = ref(false);
  const preferences = ref<FormPreference[]>([]);
  const defaultId = ref<string | null>(null);

  const defaultPreference = computed<FormPreference | null>(() => {
    if (!defaultId.value) return null;
    return preferences.value.find((p) => p.id === defaultId.value) ?? null;
  });

  const load = async (): Promise<void> => {
    isLoading.value = true;
    try {
      const entry = await UiStateService.get<FormPreferenceState>(SCOPE, formKey);
      preferences.value = entry.value.items ?? [];
      defaultId.value = entry.value.default_id ?? null;
    } catch (error) {
      if (!isApiHttpError(error) || error.status !== 404)
        notifyError(t('ui.preferenceError'), error);
      preferences.value = [];
      defaultId.value = null;
    } finally {
      isLoading.value = false;
    }
  };

  const save = async (next: FormPreferenceState): Promise<boolean> => {
    if (isSaving.value) return false;
    isSaving.value = true;
    try {
      await UiStateService.upsert<FormPreferenceState>(SCOPE, formKey, next);
      preferences.value = next.items;
      defaultId.value = next.default_id;
      return true;
    } catch (error) {
      notifyError(t('ui.presetError'), error);
      return false;
    } finally {
      isSaving.value = false;
    }
  };
  const addPreference = (name: string, fields: Record<string, unknown>): Promise<boolean> => {
    if (preferences.value.length >= MAX_PREFS) return Promise.resolve(false);
    return save({
      items: [
        ...preferences.value,
        {
          id: generateId(),
          name,
          fields: JSON.parse(JSON.stringify(fields)) as Record<string, unknown>,
        },
      ],
      default_id: defaultId.value,
    });
  };
  const updatePreference = (id: string, name: string, fields: Record<string, unknown>) =>
    save({
      items: preferences.value.map((p) => (p.id === id ? { ...p, name, fields } : p)),
      default_id: defaultId.value,
    });
  const removePreference = (id: string) =>
    save({
      items: preferences.value.filter((p) => p.id !== id),
      default_id: defaultId.value === id ? null : defaultId.value,
    });
  const setDefault = (id: string | null) => save({ items: [...preferences.value], default_id: id });
  const renamePreference = (id: string, name: string) =>
    save({
      items: preferences.value.map((p) => (p.id === id ? { ...p, name } : p)),
      default_id: defaultId.value,
    });

  onMounted(() => {
    void load();
  });

  return {
    isLoading,
    isSaving,
    preferences,
    defaultId,
    defaultPreference,
    addPreference,
    updatePreference,
    removePreference,
    setDefault,
    renamePreference,
    load,
  };
}
