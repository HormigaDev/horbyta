import { ref, computed, onMounted } from 'vue';
import { UiStateService } from 'src/services/uiState';
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
    } catch {
      preferences.value = [];
      defaultId.value = null;
    } finally {
      isLoading.value = false;
    }
  };

  const save = async (): Promise<void> => {
    isSaving.value = true;
    try {
      await UiStateService.upsert<FormPreferenceState>(SCOPE, formKey, {
        items: preferences.value,
        default_id: defaultId.value,
      });
    } catch (err) {
      notifyError('Error saving preferences', err);
    } finally {
      isSaving.value = false;
    }
  };

  const addPreference = async (name: string, fields: Record<string, unknown>): Promise<void> => {
    if (preferences.value.length >= MAX_PREFS) return;
    preferences.value = [
      ...preferences.value,
      { id: generateId(), name, fields },
    ];
    await save();
  };

  const updatePreference = async (id: string, name: string, fields: Record<string, unknown>): Promise<void> => {
    preferences.value = preferences.value.map((p) =>
      p.id === id ? { ...p, name, fields } : p,
    );
    await save();
  };

  const removePreference = async (id: string): Promise<void> => {
    preferences.value = preferences.value.filter((p) => p.id !== id);
    if (defaultId.value === id) defaultId.value = null;
    await save();
  };

  const setDefault = async (id: string | null): Promise<void> => {
    defaultId.value = id;
    await save();
  };

  const renamePreference = async (id: string, newName: string): Promise<void> => {
    preferences.value = preferences.value.map((p) =>
      p.id === id ? { ...p, name: newName } : p,
    );
    await save();
  };

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
