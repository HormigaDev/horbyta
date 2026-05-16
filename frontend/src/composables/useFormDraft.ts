import { computed, onUnmounted, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';

const DRAFT_PREFIX = 'form-draft';
const DEBOUNCE_MS = 800;

export function useFormDraft<T extends object>(
  getState: () => T,
  applyState: (draft: T) => void,
  options: { formKey: string; enabled?: () => boolean },
) {
  const $q = useQuasar();
  const { t } = useI18n();
  const route = useRoute();

  const storageKey = computed(
    () => `${DRAFT_PREFIX}:${options.formKey}:${(route.params.id as string) || 'new'}`,
  );

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const saveDraft = () => {
    if (options.enabled && !options.enabled()) return;
    try {
      localStorage.setItem(storageKey.value, JSON.stringify(getState()));
    } catch {
      // ignore
    }
  };

  const clearDraft = () => {
    try {
      localStorage.removeItem(storageKey.value);
    } catch {
      // ignore
    }
  };

  const getDraft = (): T | null => {
    try {
      const raw = localStorage.getItem(storageKey.value);
      return raw ? (JSON.parse(raw) as T) : null;
    } catch {
      return null;
    }
  };

  const scheduleSave = () => {
    if (debounceTimer !== null) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(saveDraft, DEBOUNCE_MS);
  };

  const stopAutoSave = watch(getState, scheduleSave, { deep: true });

  /**
   * Call this after initial data is loaded. Shows a dialog if a draft exists.
   * Returns true if the draft was restored, false otherwise.
   */
  const restoreDraftIfAvailable = (): Promise<boolean> => {
    const draft = getDraft();
    if (!draft) return Promise.resolve(false);

    return new Promise<boolean>((resolve) => {
      $q.dialog({
        title: t('common.draft.restoreTitle'),
        message: t('common.draft.restoreMessage'),
        ok: { label: t('common.draft.restoreAction'), color: 'primary', unelevated: true },
        cancel: { label: t('common.draft.discardAction'), flat: true },
      })
        .onOk(() => {
          applyState(draft);
          resolve(true);
        })
        .onCancel(() => {
          clearDraft();
          resolve(false);
        });
    });
  };

  onUnmounted(() => {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    stopAutoSave();
  });

  return { clearDraft, saveDraft, restoreDraftIfAvailable };
}
