<template>
  <div class="row">
    <div class="col-5">
      <q-input
        v-model.number="hours"
        type="number"
        min="0"
        max="14"
        :label="t('common.hours')"
        outlined
        dense
        @update:model-value="onInput"
        @keydown="preventInvalidChars"
        class="hide-spinners"
      />
    </div>
    <div class="col-1"></div>
    <div class="col-6">
      <q-input
        v-model.number="minutes"
        type="number"
        min="0"
        max="59"
        :label="t('common.minutes')"
        outlined
        dense
        :disable="!!(hours && +hours >= 14)"
        @update:model-value="onInput"
        @keydown="preventInvalidChars"
        class="hide-spinners"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  modelValue: number;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();

const { t } = useI18n();

const hours = ref<number | string | null>(0);
const minutes = ref<number | string | null>(0);

watch(
  () => props.modelValue,
  (newTotalMinutes) => {
    const validTotal = Math.min(newTotalMinutes || 0, 14 * 60);
    hours.value = Math.floor(validTotal / 60);
    minutes.value = validTotal % 60;
  },
  { immediate: true },
);

const preventInvalidChars = (e: KeyboardEvent) => {
  if (['-', '+', 'e', 'E', '.', ','].includes(e.key)) {
    e.preventDefault();
  }
};

const onInput = () => {
  let h = Math.max(0, Number(hours.value) || 0);
  let m = Math.max(0, Number(minutes.value) || 0);

  let forceUpdateH = false;
  let forceUpdateM = false;

  if (h >= 14) {
    h = 14;
    m = 0;
    forceUpdateH = true;
    forceUpdateM = true;
  } else if (m > 59) {
    m = 59;
    forceUpdateM = true;
  }

  if (forceUpdateH || (hours.value !== '' && hours.value !== null && Number(hours.value) < 0)) {
    hours.value = h;
  }

  if (
    forceUpdateM ||
    (minutes.value !== '' && minutes.value !== null && Number(minutes.value) < 0)
  ) {
    minutes.value = m;
  }

  emit('update:modelValue', h * 60 + m);
};
</script>

<style scoped>
:deep(.hide-spinners input[type='number']::-webkit-outer-spin-button),
:deep(.hide-spinners input[type='number']::-webkit-inner-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
</style>
