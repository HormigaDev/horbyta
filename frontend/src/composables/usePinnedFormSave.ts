import { computed, ref } from 'vue';

type MaybePromise<T> = T | Promise<T>;

interface SaveSuccessHandlers {
  reset: () => MaybePromise<unknown>;
  redirect: () => MaybePromise<unknown>;
}

export const usePinnedFormSave = () => {
  const stayOnFormAfterSave = ref(false);
  const pinButtonColor = computed(() => (stayOnFormAfterSave.value ? 'primary' : 'grey'));

  const handlePinnedSaveSuccess = async ({ reset, redirect }: SaveSuccessHandlers) => {
    if (stayOnFormAfterSave.value) {
      await reset();
      return;
    }

    await redirect();
  };

  return {
    stayOnFormAfterSave,
    pinButtonColor,
    handlePinnedSaveSuccess,
  };
};
