import { onUnmounted, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
import { useAuthStore } from 'src/stores/auth';

const DRAFT_PREFIX = 'form-draft';
const DEBOUNCE_MS = 800;

export function useFormDraft<T extends object>(
  getState: () => T,
  applyState: (draft: T) => void,
  options: { formKey: string; identity?: string; enabled?: () => boolean },
) {
  const $q = useQuasar();
  const { t } = useI18n();
  const route = useRoute();
  const auth = useAuthStore();
  let ready = false;
  let cleanSnapshot = '';
  let discardSnapshot: string | null = null;

  // Freeze identity and record at mount: leaving a route or logging out must not
  // write the old form into a different record/account's draft key.
  const storageKey = `${DRAFT_PREFIX}:${options.identity ?? auth.user?.id ?? 'anonymous'}:${options.formKey}:${(route.params.id as string) || 'new'}`;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const saveDraft = () => {
    if (!ready || (options.enabled && !options.enabled())) return;
    try {
      const value = JSON.stringify(getState());
      if (value === cleanSnapshot || value === discardSnapshot) return;
      localStorage.setItem(storageKey, value);
    } catch {
      // ignore
    }
  };

  const clearDraft = () => {
    discardSnapshot = JSON.stringify(getState());
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    try {
      localStorage.removeItem(storageKey);
    } catch {
      // ignore
    }
  };

  const getDraft = (): T | null => {
    try {
      const raw = localStorage.getItem(storageKey);
      if (!raw) return null;
      const value: unknown = JSON.parse(raw);
      return value && typeof value === 'object' && !Array.isArray(value) ? (value as T) : null;
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
    cleanSnapshot = JSON.stringify(getState());
    const draft = getDraft();
    if (!draft) {
      ready = true;
      return Promise.resolve(false);
    }

    return new Promise<boolean>((resolve) => {
      $q.dialog({
        title: t('common.draft.restoreTitle'),
        message: t('common.draft.restoreMessage'),
        ok: { label: t('common.draft.restoreAction'), color: 'primary', unelevated: true },
        cancel: { label: t('common.draft.discardAction'), flat: true },
      })
        .onOk(() => {
          ready = true;
          applyState(draft);
          resolve(true);
        })
        .onCancel(() => {
          clearDraft();
          ready = true;
          resolve(false);
        });
    });
  };

  onUnmounted(() => {
    if (debounceTimer !== null) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }
    if (ready) saveDraft();
    stopAutoSave();
  });

  return { clearDraft, saveDraft, restoreDraftIfAvailable };
}
