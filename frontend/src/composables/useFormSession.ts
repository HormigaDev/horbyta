import { ref } from 'vue';
import { useFormDraft } from 'src/composables/useFormDraft';
import { useUnsavedChanges } from 'src/composables/useUnsavedChanges';

/** Preserve drafts and warn before leaving; the caller initializes after loading the record. */
export function useFormSession<T extends object>(
  getState: () => T,
  applyState: (state: T) => void,
  options: {
    formKey: string;
    identity?: string;
    enabled: () => boolean;
    extraDirty?: () => boolean;
  },
) {
  const baseline = ref('');
  const ready = ref(false);
  const { clearDraft, restoreDraftIfAvailable } = useFormDraft(getState, applyState, {
    ...options,
    enabled: () => ready.value && options.enabled(),
  });
  useUnsavedChanges(
    () =>
      ready.value &&
      options.enabled() &&
      (baseline.value !== JSON.stringify(getState()) || !!options.extraDirty?.()),
  );
  async function initialize() {
    baseline.value = JSON.stringify(getState());
    await restoreDraftIfAvailable();
    ready.value = true;
  }
  function markSaved() {
    baseline.value = JSON.stringify(getState());
    clearDraft();
  }
  return { initialize, markSaved };
}
