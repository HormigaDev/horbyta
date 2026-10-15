import { onBeforeUnmount, onMounted, ref } from 'vue';
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

export function useUnsavedChanges(isDirty: () => boolean) {
  const $q = useQuasar();
  const { t } = useI18n();

  const resolveLeave = ref<((confirmed: boolean) => void) | null>(null);

  const promptUnsaved = (): Promise<boolean> => {
    return new Promise((resolve) => {
      $q.dialog({
        title: t('common.unsavedChanges.title'),
        message: t('common.unsavedChanges.message'),
        ok: { label: t('common.unsavedChanges.leaveAction'), color: 'negative', unelevated: true },
        cancel: { label: t('common.unsavedChanges.stayAction'), flat: true },
      })
        .onOk(() => resolve(true))
        .onCancel(() => resolve(false));
    });
  };

  onBeforeRouteLeave(async (_to, _from, next) => {
    if (!isDirty()) {
      next();
      return;
    }
    const confirmed = await promptUnsaved();
    next(confirmed ? undefined : false);
  });

  onBeforeRouteUpdate(async (to, from) => {
    if (to.path !== from.path && isDirty()) return await promptUnsaved();
    return true;
  });

  const beforeUnloadHandler = (event: BeforeUnloadEvent) => {
    if (!isDirty()) return;
    event.preventDefault();
  };

  onMounted(() => {
    window.addEventListener('beforeunload', beforeUnloadHandler);
  });

  onBeforeUnmount(() => {
    window.removeEventListener('beforeunload', beforeUnloadHandler);
    if (resolveLeave.value) {
      resolveLeave.value(false);
      resolveLeave.value = null;
    }
  });
  return { confirmLeave: () => (isDirty() ? promptUnsaved() : Promise.resolve(true)) };
}
